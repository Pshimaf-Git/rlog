use rlog::{
    Logger, 
    Rlogger
};

fn main() {
    let mut logger = Rlogger::default();

    logger.debug("This is a debug message from main.");
    logger.info("This is an info message from main.");
    logger.warn("This is a warning message from main.");
    logger.error("This is an error message from main.");
    // logger.fatal("This is a fatal message from main."); // This would exit the program
}
