use crate::{parser::Term, typing::Type};

trait MutVisitor: Sized {
    fn visit_var(&mut self, var: &mut Term) {}

    fn visit_const(&mut self, t: &mut Term) {}

    fn visit_succ(&mut self, t: &mut Term) {
        self.visit_term(t);
    }

    fn visit_abs(&mut self, body: &mut Term) {
        self.visit_term(body);
    }

    fn visit_app(&mut self, t1: &mut Term, t2: &mut Term) {
        self.visit_term(t1);
        self.visit_term(t2);
    }

    fn visit_if(&mut self, guard: &mut Term, csq: &mut Term, alt: &mut Term) {
        self.visit_term(guard);
        self.visit_term(csq);
        self.visit_term(alt);
    }

    fn visit_term(&mut self, term: &mut Term) {
        walk_mut_term(self, term);
    }
}

fn walk_mut_term<V: MutVisitor>(visitor: &mut V, var: &mut Term) {
    match var {
        Term::TmTrue | Term::TmFalse | Term::TmZero => visitor.visit_const(var),
        Term::TmSucc(t) => visitor.visit_succ(t),
        Term::TmVar(_) => visitor.visit_var(var),
        Term::TmAbs(_, _ty, body) => visitor.visit_abs(body),
        Term::TmApp(t1, t2) => visitor.visit_app(t1, t2),
        Term::TmIf(a, b, c) => visitor.visit_if(a, b, c),
    }
}

#[derive(Copy, Clone, Debug)]
enum Direction {
    Up,
    Down,
}

#[derive(Copy, Clone, Debug)]
struct Shifting {
    cutoff: usize,
    direction: Direction,
}

impl Default for Shifting {
    fn default() -> Self {
        Shifting {
            cutoff: 0,
            direction: Direction::Up,
        }
    }
}

impl Shifting {
    pub fn new(direction: Direction) -> Self {
        Shifting {
            cutoff: 0,
            direction,
        }
    }
}

impl MutVisitor for Shifting {
    fn visit_var(&mut self, var: &mut Term) {
        let n = match var {
            Term::TmVar(n) => n,
            _ => unreachable!(),
        };

        if *n >= self.cutoff {
            match self.direction {
                Direction::Up => *n += 1,
                Direction::Down => *n -= 1,
            }
        }
    }

    fn visit_abs(&mut self, body: &mut Term) {
        self.cutoff += 1;
        self.visit_term(body);
        self.cutoff -= 1;
    }
}

#[derive(Debug)]
struct Substitution {
    cutoff: usize,
    term: Term,
}

impl Substitution {
    pub fn new(term: Term) -> Substitution {
        Substitution { cutoff: 0, term }
    }
}

impl MutVisitor for Substitution {
    fn visit_var(&mut self, var: &mut Term) {
        match var {
            Term::TmVar(n) if *n >= self.cutoff => {
                *var = self.term.clone();
            }
            _ => unreachable!(),
        }
    }

    fn visit_abs(&mut self, body: &mut Term) {
        self.cutoff += 1;
        walk_mut_term(self, body);
        self.cutoff -= 1;
    }
}

pub fn substitution(mut val: Term, body: &mut Term) {
    Shifting::new(Direction::Up).visit_term(&mut val);
    Substitution::new(val).visit_term(body);
    Shifting::new(Direction::Down).visit_term(body);
}
