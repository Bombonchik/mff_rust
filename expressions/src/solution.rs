use std::rc::Rc;
use std::collections::HashMap;
pub trait Expression {
    fn accept(&self, visitor: &mut dyn Visitor);
    fn as_ref(&self) -> &dyn Expression;
    fn rc_clone(&self) -> Rc<dyn Expression>;
}


#[derive(Clone)]
pub struct Const {
    value: i32,
}

#[derive(Clone)]
pub struct Variable {
    name: String,
}

impl Const {
    pub fn new(value: i32) -> Self {
        Const { value }
    }
    pub fn value(&self) -> i32 { self.value }
    pub fn as_ref(&self) -> &dyn Expression {
        self
    }
}

impl Expression for Const {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_const(self);
    }
    fn as_ref(&self) -> &dyn Expression {
        self
    }
    fn rc_clone(&self) -> Rc<dyn Expression> {
        Rc::new(self.clone())
    }
}

impl Variable {
    pub fn new(name: String) -> Self {
        Variable { name }
    }
    pub fn name(&self) -> &str { &self.name }
    pub fn as_ref(&self) -> &dyn Expression {
        self
    }
}

impl Expression for Variable {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_var(self);
    }
    fn as_ref(&self) -> &dyn Expression {
        self
    }
    fn rc_clone(&self) -> Rc<dyn Expression> {
        Rc::new(self.clone())
    }
}


#[derive(Clone)]
pub struct Sum {
    left: Rc<dyn Expression>,
    right: Rc<dyn Expression>,
}

impl Sum {
    pub fn new<L: Expression + 'static, R: Expression + 'static>(left: L, right: R) -> Self {
        Sum {
            left: Rc::new(left),
            right: Rc::new(right),
        }
    }
    pub fn left(&self) -> &dyn Expression { self.left.as_ref() }
    pub fn right(&self) -> &dyn Expression { self.right.as_ref() }
    pub fn as_ref(&self) -> &dyn Expression {
        self
    }
}


impl Expression for Sum {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_sum(self);
    }
    fn as_ref(&self) -> &dyn Expression {
        self
    }
    fn rc_clone(&self) -> Rc<dyn Expression> {
        Rc::new(self.clone())
    }
}


#[derive(Clone)]
pub struct Product {
    left: Rc<dyn Expression>,
    right: Rc<dyn Expression>,
}

impl Product {
    pub fn new<L: Expression + 'static, R: Expression + 'static>(left: L, right: R) -> Self {
        Self {
            left: Rc::new(left),
            right: Rc::new(right),
        }
    }
    pub fn left(&self) -> &dyn Expression { self.left.as_ref() }
    pub fn right(&self) -> &dyn Expression { self.right.as_ref() }
    pub fn as_ref(&self) -> &dyn Expression {
        self
    }
}


impl Expression for Product {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_product(self);
    }
    fn as_ref(&self) -> &dyn Expression {
        self
    }
    fn rc_clone(&self) -> Rc<dyn Expression> {
        Rc::new(self.clone())
    }
}

pub struct PostfixConvertor {
    result: Vec<String>,
}

pub trait Visitor {
    fn visit_const(&mut self, cst: &Const);
    fn visit_var(&mut self, var: &Variable);
    fn visit_sum(&mut self, sum: &Sum);
    fn visit_product(&mut self, product: &Product);
}


impl PostfixConvertor {
    pub fn transform(expression: &dyn Expression) -> String {
        let mut visitor = Self::new();
        expression.accept(&mut visitor);
        visitor.consume()
    }
    fn new() -> Self {
        PostfixConvertor { result: Vec::new() }
    }
    fn consume(self) -> String { self.result.join(" ") }
}

impl Visitor for PostfixConvertor {
    fn visit_const(&mut self, cst: &Const) {
        self.result.push(format!("{}", cst.value()));
    }

    fn visit_var(&mut self, var: &Variable) {
        self.result.push(format!("{}", var.name()));
    }

    fn visit_sum(&mut self, sum: &Sum) {
        sum.left().accept(self);
        sum.right().accept(self);
        self.result.push(format!("+"));
    }

    fn visit_product(&mut self, product: &Product) {
        product.left().accept(self);
        product.right().accept(self);
        self.result.push(format!("*"));
    }
}

pub struct Evaluate<'a> {
    result: Result<i32, String>,
    values: &'a HashMap<String, i32>,
}

impl<'a> Evaluate<'a> {
    pub fn transform(expression: &dyn Expression, values: &'a HashMap<String, i32>) -> Result<i32, String> {
        let mut evaluator = Evaluate { result: Ok(0), values };
        expression.accept(&mut evaluator);
        evaluator.result
    }
}

impl<'a> Visitor for Evaluate<'a> {
    fn visit_const(&mut self, cst: &Const) {
        if let Ok(ref mut result) = self.result {
            *result = cst.value(); 
        }
    }

    fn visit_var(&mut self, var: &Variable) {
        if let Ok(ref mut result) = self.result {
            *result = match self.values.get(var.name()) { 
                Some(&value) => value,
                None => {
                    self.result = Err(format!("Missing variable {}", var.name()));
                    return;
                }
            };
        }
    }
    fn visit_sum(&mut self, sum: &Sum) {
        if self.result.is_err() {
            return;
        }
    
        sum.left().accept(self);
        let left = match &self.result {
            Ok(value) => *value,
            Err(_) => return,
        };
    
        sum.right().accept(self);
        let right = match &self.result {
            Ok(value) => *value,
            Err(_) => return,
        };
    
        self.result = Ok(left + right);
    }

    fn visit_product(&mut self, product: &Product) {
        if self.result.is_err() {
            return;
        }
    
        product.left().accept(self);
        let left = match &self.result {
            Ok(value) => *value,
            Err(_) => return,
        };
    
        product.right().accept(self);
        let right = match &self.result {
            Ok(value) => *value,
            Err(_) => return,
        };
    
        self.result = Ok(left * right);
    }
}


pub struct Substitute<'a> {
    result: Option<Rc<dyn Expression>>,
    values: &'a HashMap<String, &'a dyn Expression>,
}

impl<'a> Substitute<'a> {
    pub fn transform(expression: &'a dyn Expression, values: &'a HashMap<String, &'a dyn Expression>) -> Rc<dyn Expression> {
        let mut substitute  = Substitute { result: None, values };
        expression.accept(&mut substitute);
        substitute.result.unwrap()
    }
}

impl<'a> Visitor for Substitute<'a> {
    fn visit_const(&mut self, cst: &Const) {
        self.result = Some(Rc::new(cst.clone()));
    }

    fn visit_var(&mut self, var: &Variable) {
        if let Some(&expr) = self.values.get(&var.name) {
            self.result = Some((*expr).rc_clone());
        } else {
            self.result = Some(Rc::new(var.clone()));
        }
    }
    fn visit_sum(&mut self, sum: &Sum) {
        sum.left().accept(self);
        let left = self.result.clone().unwrap();
    
        sum.right().accept(self);
        let right = self.result.clone().unwrap();
        self.result = Some(Rc::new(Sum{left, right}));
    }

    fn visit_product(&mut self, product: &Product) {
        product.left().accept(self);
        let left = self.result.clone().unwrap();
    
        product.right().accept(self);
        let right = self.result.clone().unwrap();
        self.result = Some(Rc::new(Product{left, right}));
    }
}