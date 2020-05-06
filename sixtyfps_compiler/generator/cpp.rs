mod cpp_ast {

    use std::cell::Cell;
    use std::fmt::{Display, Error, Formatter};
    thread_local!(static INDETATION : Cell<u32> = Cell::new(0));
    fn indent(f: &mut Formatter<'_>) -> Result<(), Error> {
        INDETATION.with(|i| {
            for _ in 0..(i.get()) {
                write!(f, "    ")?;
            }
            Ok(())
        })
    }

    #[derive(Default, Debug)]
    pub struct File {
        pub includes: Vec<String>,
        pub declarations: Vec<Declaration>,
    }

    impl Display for File {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            for i in &self.includes {
                writeln!(f, "#include {}", i)?;
            }
            for d in &self.declarations {
                write!(f, "\n{}", d)?;
            }
            Ok(())
        }
    }

    #[derive(Debug, derive_more::Display)]
    pub enum Declaration {
        Struct(Struct),
        Function(Function),
        Var(Var),
    }

    #[derive(Default, Debug)]
    pub struct Struct {
        pub name: String,
        pub members: Vec<Declaration>,
    }

    impl Display for Struct {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            indent(f)?;
            writeln!(f, "struct {} {{", self.name)?;
            INDETATION.with(|x| x.set(x.get() + 1));
            for m in &self.members {
                // FIXME! identation
                write!(f, "{}", m)?;
            }
            INDETATION.with(|x| x.set(x.get() - 1));
            indent(f)?;
            writeln!(f, "}};")
        }
    }

    #[derive(Default, Debug)]
    pub struct Function {
        pub name: String,
        /// (...) -> ...
        pub signature: String,
        pub is_constructor: bool,
        pub statements: Vec<String>,
    }

    impl Display for Function {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            indent(f)?;
            if !self.is_constructor {
                write!(f, "auto ")?;
            }
            writeln!(f, "{} {} {{", self.name, self.signature)?;
            for s in &self.statements {
                indent(f)?;
                writeln!(f, "    {}", s)?;
            }
            indent(f)?;
            writeln!(f, "}}")
        }
    }

    #[derive(Default, Debug)]
    pub struct Var {
        pub ty: String,
        pub name: String,
        pub init: Option<String>,
    }

    impl Display for Var {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            indent(f)?;
            write!(f, "{} {}", self.ty, self.name)?;
            if let Some(i) = &self.init {
                write!(f, " = {}", i)?;
            }
            writeln!(f, ";")
        }
    }
}

pub fn generate(component: &crate::lower::LoweredComponent) -> impl std::fmt::Display {
    use cpp_ast::*;
    let mut x = File::default();

    x.includes.push("<sixtyfps.h>".into());

    x.declarations.push(Declaration::Struct(Struct {
        name: component.id.clone(),
        members: vec![
            Declaration::Var(Var {
                ty: format!("sixtyfps::{}", component.root_item.native_type.class_name),
                name: "root".to_owned(),
                ..Default::default()
            }),
            Declaration::Function(Function {
                name: component.id.clone(),
                signature: "()".to_owned(),
                is_constructor: true,
                statements: component
                    .root_item
                    .init_properties
                    .iter()
                    .map(|(s, i)| format!("root.{} = \"{}\";", s, i))
                    .collect(),
            }),
        ],
    }));

    x.declarations.push(Declaration::Var(Var {
        ty: "sixtyfps::ItemTreeNode".to_owned(),
        name: format!("{}_children[]", component.id),
        init: Some(format!(
            "{{ sixtyfps::ItemTreeNode{{0, &sixtyfps::{}, 0, 0}} }}",
            component.root_item.native_type.vtable
        )),
    }));

    x.declarations.push(Declaration::Function(Function {
        name: "main".into(),
        signature: "() -> int".to_owned(),
        is_constructor: false,
        statements: vec![
            format!("{} component;", component.id),
            format!("static const sixtyfps::ComponentType componentType{{ nullptr, nullptr, [](const sixtyfps::ComponentType*) -> const sixtyfps::ItemTreeNode* {{return {}_children; }}  }};", component.id),
            format!("sixtyfps::run(&component, &componentType);"),
        ],
    }));
    x
}
