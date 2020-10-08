use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use bastion::prelude::*;

fn main() {
    Bastion::init();

    let started = AtomicBool::new(false);
    let started = Arc::new(started);

    Bastion::children(|children| {
        children.with_exec(move |ctx: BastionContext| {
            let started = started.clone();
            async move {
                println!("Started!");

                if started.swap(true, Ordering::SeqCst) {
                    println!("Already started once. Stopping...");

                    // This will ask the system to stop itself...
                    Bastion::stop();
                    // ...and this will stop this child immediately...
                    return Ok(());
                    // Note that if Err(()) was returned, the child would have been
                    // restarted (and if the system wasn't stopping).
                }

                // This will return None.
                let try_recv = ctx.try_recv().await;
                println!("try_recv.is_some() == {}", try_recv.is_some()); // false

                let current = ctx.current();

                let answer = ctx
                    .ask(&current.addr(), "Hello World!")
                    .expect("Couldn't send the message.");

                let message = ctx.recv().await?;

                msg! { message,
                                                    msg: &'static str =!> {
                                                        println!(r#"msg == "Hello World!" => {}"#, msg == "Hello World!"); // true
                                                        answer!(message, "Goodbye!");

                //test will

                                /*                        let (mut msg, sign) = message.extract();
                                                        let sender = msg.take_sender().expect("failed to take render");
                                                        sender.send("Goodbye!", sign);
                                */
                                                    };
                                                    // This won't happen because this example
                                                    // only "asks" a &'static str...
                                                    _: _ => ();
                                                }

                msg! { answer.await?,
                    msg: &'static str => {
                        println!(r#"msg == "Goodbye!" => {}"#, msg == "Goodbye!"); // true
                    };
                    // This won't happen because this example
                    // only answers a &'static str...
                    _: _ => ();
                }

                // Panicking will restart the children group.
                panic!("Oh no!");
            }
        })
    })
    .expect("Couldn't start a new children group.");

    Bastion::start();
    Bastion::block_until_stopped();
}
