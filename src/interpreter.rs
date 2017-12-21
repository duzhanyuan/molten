
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use parser::AST;
use scope::{ Scope, ScopeRef };
use types::Type;


#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Integer(isize),
    Real(f64),
    String(String),
    List(Vec<Value>),
    Function(ScopeRef, Vec<(String, Option<Type>, Option<AST>)>, AST),
    Builtin(fn(ScopeRef, Vec<Value>) -> Value),
    AST(AST),
}

impl Value {
    fn is_true(&self) -> bool {
        self != &Value::Nil && self != &Value::Boolean(false)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Value::Function(ref fscope, ref args, ref body) => write!(f, "Function({:?})", body),
            _ => write!(f, "{:?}", self)
        }
    }
}



#[derive(Clone, Debug, PartialEq)]
pub struct Env {
    pub names: HashMap<String, Value>,
    pub parent: Option<EnvRef>,
}

pub type EnvRef = Rc<RefCell<Env>>;


impl Env {
    pub fn new(parent: Option<EnvRef>) -> Env {
        Env {
            names: HashMap::new(),
            parent: parent,
        }
    }

    pub fn new_ref(parent: Option<EnvRef>) -> EnvRef {
        Rc::new(RefCell::new(Env::new(parent)))
    }

    pub fn set_parent(&mut self, parent: EnvRef) {
        self.parent = Some(parent);
    }

    pub fn assign(&mut self, name: String, value: Value) {
        //match self.names.entry(name.clone()) {
        //    Entry::Vacant(_) => panic!("NameError: variable is undefined; {:?}", name),
        //    Entry::Occupied(mut entry) => Rc::get_mut(entry.get_mut()).unwrap().value = Some(value),
        //}
        match self.names.contains_key(&name) {
            true => panic!("NameError: variable is already defined; {:?}", name),
            false => self.names.insert(name, value)
        };
    }

    pub fn find(&self, name: &String) -> Option<Value> {
        if let Some(x) = self.names.get(name) {
            return Some(x.clone());
        }
        else if let Some(ref parent) = self.parent {
            return parent.borrow().find(name).map(|x| x.clone());
        }
        else {
            return None;
        }
    }
}





pub fn execute(scope: ScopeRef, code: &Vec<AST>) {
    let mut machine = Interpreter::new();
    builtins::assign_builtins(scope.borrow().get_parent().unwrap().clone());
    let result = machine.execute_vec(scope, code);

}

pub struct Interpreter {

}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {

        }
    }

    pub fn execute_vec(&mut self, scope: ScopeRef, code: &Vec<AST>) -> Value {
        let mut last: Value = Value::Nil;

        for node in code {
            last = self.execute_node(scope.clone(), node);
            println!("PARTIAL: {}", last);
        }
        last
    }

    pub fn execute_node(&mut self, scope: ScopeRef, node: &AST) -> Value {
        match *node {
            AST::Nil => Value::Nil,
            AST::Boolean(boolean) => Value::Boolean(boolean),
            AST::Integer(num) => Value::Integer(num),
            AST::Real(num) => Value::Real(num),
            AST::String(ref string) => Value::String(string.clone()),

            AST::List(ref items) => {
                let mut list = vec!();
                for item in items {
                    list.push(self.execute_node(scope.clone(), item));
                }
                Value::List(list)
            },

            AST::Identifier(ref name) => {
                match scope.borrow().find(name).unwrap().value.clone() {
                    None => panic!("UnsetError: use before assignment {:?}", name),
                    Some(x) => x
                }
            },

            AST::Invoke(ref name, ref args) => {
                let func = &scope.borrow().find(name).unwrap().value;

                let mut values = vec!();
                for arg in args {
                    values.push(self.execute_node(scope.clone(), arg));
                }

                if let &Some(Value::Function(ref fscope, ref argdefs, ref body)) = func {
                    let lscope = Scope::new_ref(fscope.borrow().get_parent());
                    for (ref def, value) in argdefs.iter().zip(values.iter()) {
                        lscope.borrow_mut().define(def.0.clone(), def.1.clone());
                        lscope.borrow_mut().assign(&def.0, value.clone());
                    }

                    self.execute_node(lscope.clone(), body)
                }
                else if let &Some(Value::Builtin(ref funcptr)) = func {
                    funcptr(scope.clone(), values)
                }
                else {
                    panic!("Function not found: {:?}", name);
                }
            },

            AST::Function(ref args, ref body, ref fscope, _, _) => {
                Value::Function(fscope.clone(), args.clone(), *body.clone())
            },

            AST::Definition((ref name, ref ttype), ref body) => {
                let value = self.execute_node(scope.clone(), body);
                scope.borrow_mut().assign(name, value.clone());
                value
            },

            AST::Block(ref body) => { self.execute_vec(scope.clone(), body) },

            AST::If(ref cond, ref texpr, ref fexpr) => {
                let result = self.execute_node(scope.clone(), cond);
                if result.is_true() {
                    self.execute_node(scope.clone(), texpr)
                }
                else {
                    self.execute_node(scope.clone(), fexpr)
                }
            },

    /*
            AST::Raise(ref expr) => {

            },

            AST::Try(ref cond, ref cases) => {

            },

            AST::For(ref name, ref cond, ref body, ref lscope) => {

            },
    */

    /*
            AST::Match(ref cond, ref cases) => {
                let old = self.indent.clone();
                self.indent = old.clone() + &"    ";

                // TODO should you implement this as an if statement instead?
                let mut compiled_cases = vec!();
                for &(ref case, ref expr) in cases {
                    compiled_cases.push(format!("{space}  case {}:\n{indent}{}\n{indent}break;", self.execute_node(scope.clone(), case), self.execute_node(scope.clone(), expr), space=old, indent=self.indent));
                }
                let compiled = format!("select ({}) {{\n{}\n{space}}}", self.execute_node(scope.clone(), cond), compiled_cases.join("\n"), space=old);

                self.indent = old;
                compiled
            },

            AST::While(ref cond, ref body) => {
                let old = self.indent.clone();
                self.indent = old.clone() + &"    ";
                let compiled = format!("while ({}) {{\n{indent}{}\n{space}}}", self.execute_node(scope.clone(), cond), add_terminator(self.execute_node(scope.clone(), body)), space=old, indent=self.indent);
                self.indent = old;
                compiled
            },

            AST::Class(ref name, ref body, ref cscope) => {

            },

            AST::Index(ref base, ref index) => {

            },

            AST::Accessor(ref left, ref right) => {

            },

            AST::Import(_) => { },

            AST::Type(_) => { },
    */

            _ => { Value::Nil },
        }
    }

}


mod builtins {
    use scope::ScopeRef;
    use interpreter::Value;

    pub fn assign_builtins(scope: ScopeRef) {
        let mut scope = scope.borrow_mut();
        scope.assign(&String::from("*"), Value::Builtin(multiply));
        scope.assign(&String::from("/"), Value::Builtin(divide));
        scope.assign(&String::from("+"), Value::Builtin(add));
        scope.assign(&String::from("-"), Value::Builtin(subtract));

        scope.assign(&String::from("=="), Value::Builtin(equals));
        scope.assign(&String::from("!="), Value::Builtin(not_equals));
        scope.assign(&String::from(">"), Value::Builtin(greater_than));
        scope.assign(&String::from("<"), Value::Builtin(less_than));

        //let ops = vec!("*", "/", "%", "+", "-", "<<", ">>", "<", ">", "<=", ">=", "==", "!=", "&", "|", "and", "or", "not");

    }

    macro_rules! for_numbers {
        ( $args:ident, $a:ident, $b:ident, $x:expr, $t:path ) => {
            match (&$args[0], &$args[1]) {
                (&Value::Integer(ref $a), &Value::Integer(ref $b)) => $t($x),
                (&Value::Real(ref $a), &Value::Real(ref $b)) => $t($x),
                _ => panic!("RuntimeError: expected Integer or Real value {:?} {:?}", $args[0], $args[1]),
            }
        };

        ( $args:ident, $a:ident, $b:ident, $x:expr ) => {
            match (&$args[0], &$args[1]) {
                (&Value::Integer(ref $a), &Value::Integer(ref $b)) => Value::Integer($x),
                (&Value::Real(ref $a), &Value::Real(ref $b)) => Value::Real($x),
                _ => panic!("RuntimeError: expected Integer or Real value {:?} {:?}", $args[0], $args[1]),
            }
        }
    }

    fn multiply(scope: ScopeRef, args: Vec<Value>) -> Value {
        for_numbers!(args, a, b, a * b)
    }

    fn divide(scope: ScopeRef, args: Vec<Value>) -> Value {
        for_numbers!(args, a, b, a / b)
    }

    fn add(scope: ScopeRef, args: Vec<Value>) -> Value {
        for_numbers!(args, a, b, a + b)
    }

    fn subtract(scope: ScopeRef, args: Vec<Value>) -> Value {
        for_numbers!(args, a, b, a - b)
    }

    fn equals(scope: ScopeRef, args: Vec<Value>) -> Value {
        for_numbers!(args, a, b, a == b, Value::Boolean)
    }

    fn not_equals(scope: ScopeRef, args: Vec<Value>) -> Value {
        for_numbers!(args, a, b, a != b, Value::Boolean)
    }

    fn greater_than(scope: ScopeRef, args: Vec<Value>) -> Value {
        for_numbers!(args, a, b, a > b, Value::Boolean)
    }

    fn less_than(scope: ScopeRef, args: Vec<Value>) -> Value {
        for_numbers!(args, a, b, a < b, Value::Boolean)
    }
}

