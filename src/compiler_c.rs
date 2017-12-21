
use parser::AST;
use scope::ScopeRef;
use types::Type;

pub fn compile(scope: ScopeRef, code: &Vec<AST>) -> String {
    let mut data = CompilerData::new();
    let main = data.compile_vec(scope, code, true);

    let mut compiled = String::new();
    for func in data.funcs {
        compiled.push_str(func.as_str());
        compiled.push_str("\n\n");
    }
    compiled.push_str(format!("int main(int argc, void **argv) {{\n{}\n}}", main).as_str());
    compiled
}

struct CompilerData {
    next: i32,
    indent: String,
    funcs: Vec<String>,
}

impl CompilerData {
    pub fn new() -> CompilerData {
        CompilerData {
            next: 1,
            indent: String::from("    "),
            funcs: vec!(),
        }
    }

    pub fn compile_vec(&mut self, scope: ScopeRef, code: &Vec<AST>, is_last: bool) -> String {
        let mut compiled = String::new();

        for i in 0 .. code.len() - 1 {
            compiled = compiled + &self.indent + &self.compile_node(scope.clone(), &code[i], if is_last && i == code.len() { true } else { false });
            if i != code.len() {
                compiled = compiled + &";\n";
            }
        }
        compiled
    }

    pub fn compile_node(&mut self, scope: ScopeRef, node: &AST, is_last: bool) -> String {
        let text = match *node {
            AST::Nil => String::from("NULL"),
            AST::Boolean(ref boolean) => match *boolean { true => String::from("1"), false => String::from("0") },
            AST::Integer(ref num) => num.to_string(),
            AST::Real(ref num) => num.to_string(),
            AST::String(ref string) => string.clone(),

            //AST::List(ref items) => {

            //},

            AST::Identifier(ref name) => { name.clone() },

            AST::Invoke(ref name, ref args) => {
                //let mut ftype = match scope.borrow().find(name) {
                //    None => panic!("Function not found: {:?}", name),
                //    Some(sym) => sym.ttype.clone(),
                //};

                /*
                if let Type::Function(ref mut types, ref ret) = ftype {
                    for (ttype, ref mut value) in types.iter().zip(args.iter()) {

                    }


                }
                else if let Type::Variable(ref name) = ftype {

                }
                else {
                    panic!("Not a function: {:?}", name);
                }
                */

                let mut compiled = vec!();
                for code in args {
                    compiled.push(self.compile_node(scope.clone(), code, false));
                }

                if let Some(result) = CompilerData::compile_builtin(name, &compiled) {
                    result
                }
                else {
                    format!("{}({})", name, compiled.join(", "))
                }
            },

            AST::Function(ref args, ref body, ref fscope, ref ftype, _) => {
                let mut compiled_args = vec!();
                for &(ref name, ref ttype, ref value) in args {
                    compiled_args.push(self.compile_variable_def(fscope.clone(), ttype.clone().unwrap(), name));
                }
                let compiled = self.compile_node(fscope.clone(), body, true);

                let name = self.next;
                self.next += 1;
                self.funcs.push(format!("{} func{}({}) {{\n{}\n}}", "int", name, compiled_args.join(", "), compiled));
                format!("func{}", name)
            },

            AST::Definition((ref name, ref ttype), ref body) => {
                self.compile_variable_def(scope.clone(), ttype.clone().unwrap(), name) + &" = " + &self.compile_node(scope.clone(), body, false)
            },

            AST::Block(ref body) => { self.compile_vec(scope.clone(), body, is_last) },

            AST::If(ref cond, ref texpr, ref fexpr) => {
                let old = self.indent.clone();
                self.indent = old.clone() + &"    ";

                let compiled_cond = self.compile_node(scope.clone(), cond, false);
                let compiled_texpr = self.compile_node(scope.clone(), texpr, is_last);
                let compiled_fexpr = self.compile_node(scope.clone(), fexpr, is_last);
                let compiled = format!("{space}if ({}) {{\n{indent}{}\n{space}}} else {{\n{indent}{}\n{space}}}", compiled_cond, compiled_texpr, compiled_fexpr, space=old, indent=self.indent);

                self.indent = old;
                compiled
            },

    /*
            AST::Raise(ref expr) => {

            },

            AST::Try(ref cond, ref cases) => {

            },

            AST::For(ref name, ref cond, ref body, ref lscope) => {

            },
    */

            AST::Match(ref cond, ref cases) => {
                let old = self.indent.clone();
                self.indent = old.clone() + &"    ";

                // TODO should you implement this as an if statement instead?
                let mut compiled_cases = vec!();
                for &(ref case, ref expr) in cases {
                    compiled_cases.push(format!("{space}  case {}:\n{indent}{}\n{indent}break;", self.compile_node(scope.clone(), case, false), self.compile_node(scope.clone(), expr, is_last), space=old, indent=self.indent));
                }
                let compiled = format!("switch ({}) {{\n{}\n{space}}}", self.compile_node(scope.clone(), cond, false), compiled_cases.join("\n"), space=old);

                self.indent = old;
                compiled
            },

            AST::While(ref cond, ref body) => {
                let old = self.indent.clone();
                self.indent = old.clone() + &"    ";
                let compiled = format!("while ({}) {{\n{indent}{}\n{space}}}", self.compile_node(scope.clone(), cond, false), add_terminator(self.compile_node(scope.clone(), body, false)), space=old, indent=self.indent);
                self.indent = old;
                compiled
            },

    /*
            AST::Class(ref name, ref body, ref cscope) => {

            },

            AST::Index(ref base, ref index) => {

            },

            AST::Accessor(ref left, ref right) => {

            },

            AST::Import(_) => { },

            AST::Type(_) => { },
    */

            _ => { String::new() },
        };

        match is_last {
            true => format!("return {};", text),
            false => text
        }
    }

    fn compile_variable_def(&mut self, scope: ScopeRef, ttype: Type, name: &String) -> String {
        // TODO finish this
        match ttype {
            Type::Concrete(ref tname) => format!("{} {}", match tname.as_str() {
                "Bool" => String::from("int"),
                "Int" => String::from("int"),
                "Real" => String::from("double"),
                "String" => String::from("char *"),
                _ => String::from("UNKNOWN"),
            }, name),
            Type::Function(ref args, ref ret) => {
                let mut compiled_args = vec!();
                for ttype in args {
                    compiled_args.push(self.compile_variable_def(scope.clone(), ttype.clone(), &String::from("")));
                }
                let compiled_ret = self.compile_variable_def(scope.clone(), *ret.clone(), &String::from(""));
                format!("{} (*{})({})", compiled_ret, name, compiled_args.join(", "))
            },
            // TODO this is temporary
            Type::Variable(ref tname) => String::from("int ") + name,
            _ => String::from("UNKNOWN ") + name,
        }
    }

    fn compile_builtin(name: &String, args: &Vec<String>) -> Option<String> {
        let ops = vec!("*", "/", "%", "+", "-", "<<", ">>", "<", ">", "<=", ">=", "==", "!=", "&", "|");

        // TODO should you pass in the arguments?
        if ops.contains(&name.as_str()) {
            Some(format!("({} {} {})", args[0], name, args[1]))
            //Some(name.clone())
        }
        else {
            match name.as_str() {
                "and" => Some(format!("({} {} {})", args[0], "&&", args[1])),
                "or" => Some(format!("({} {} {})", args[0], "||", args[1])),
                "not" => Some(format!("{}({})", "!", args[0])),
                _ => None
            }
        }
    }
}


fn add_terminator(mut text: String) -> String {
    if text.char_indices().rev().nth(1).unwrap().1 != ';' {
        text.push(';');
    }
    text
}

