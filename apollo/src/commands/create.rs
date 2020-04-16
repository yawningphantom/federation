use crate::commands::{Command, CreateGraph};

impl Command for CreateGraph {
    fn run(&self) {
        println!("Chose a name for your graph. This name is a\
        permanent identifier for your graph, and start with a lowercase letter")
    }
}
