pub struct Target<'a> {
    pub name: &'a str,
    pub dependences: &'a [&'a str],
    pub commands: &'a [Box<dyn Runable<'a>>],
}

pub trait Runable<'a> {
    fn run(self);
}
