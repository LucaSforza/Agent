use crate::problem::{Problem, WithSolution};
enum Domain {
    Int(i32),
    Float(f64),
    Str(String),
}

pub trait Variable {
    type IndexVar;

    fn set_val(&mut self, i: Self::IndexVar, val: Domain);
    fn get_val(&self, i: Self::IndexVar) -> Domain;
}

pub trait ConstraintSatisfaction: Problem + WithSolution {}
pub trait CSPAlgoritm {
    fn resolve();
}
pub struct BackTracking<P: WithSolution> {
    stack: Vec<P::Action>,
}

impl<P> CSPAlgoritm for BackTracking<P>
where
    P: WithSolution,
{
    fn resolve() {
        todo!()
    }
}
