use clap::Parser;
use crossterm::{
    cursor::{Hide, MoveToColumn, MoveUp, Show},
    execute, queue,
    style::Print,
    terminal::{Clear, ClearType, DisableLineWrap, EnableLineWrap},
    Result,
};
use float_ord::FloatOrd;
use std::collections::BTreeSet;
use std::io::{self, Write};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    signal,
    sync::mpsc,
    time,
};

mod parser;

async fn update_screen(mut rx: mpsc::Receiver<String>) {
    let mut stderr = io::stderr();
    execute!(stderr, DisableLineWrap, Hide).unwrap();

    let mut have_output = false;
    while let Some(s) = rx.recv().await {
        if have_output {
            queue!(stderr, MoveToColumn(1), Clear(ClearType::FromCursorDown,)).unwrap();
        }
        let line_count = s.lines().count();
        queue!(
            stderr,
            Print(&s),
            MoveUp(line_count.try_into().unwrap()),
            MoveToColumn(1),
        )
        .unwrap();
        stderr.flush().unwrap();
        have_output = true;
    }

    if have_output {
        queue!(stderr, MoveToColumn(1), Clear(ClearType::FromCursorDown)).unwrap();
        stderr.flush().unwrap();
    }
    execute!(stderr, EnableLineWrap, Show).unwrap();
}

fn take(it: impl DoubleEndedIterator<Item = String>, num: usize, reverse: bool) -> String {
    if reverse {
        return it.rev().take(num).collect();
    } else {
        return it.take(num).collect();
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// show this number of items while sorting
    #[clap(short, long, default_value_t = 10)]
    limit: usize,

    /// compare according to string numerical value
    #[clap(short, long)]
    numeric: bool,

    /// compare human readable numbers (e.g., 2K 1G)
    #[clap(short, long)]
    human_numeric: bool,

    /// reverse the result of comparisons
    #[clap(short, long)]
    reverse: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    let limit = args.limit;
    let reverse = args.reverse;
    let numeric = args.numeric;
    let human_numeric = args.human_numeric;

    let (tx, rx) = mpsc::channel::<String>(16);
    let handle = tokio::spawn(update_screen(rx));

    let mut interval = time::interval(time::Duration::from_millis(250));
    let mut lines: BTreeSet<(FloatOrd<f64>, String, usize)> = BTreeSet::new();
    let mut num: usize = 0;
    let mut buf = String::new();
    let mut reader = BufReader::new(tokio::io::stdin());
    let mut dirty = false;
    loop {
        tokio::select! {
            _ = signal::ctrl_c() => break,
            _ = interval.tick() => {
                if dirty {
                    let mut t = take(lines.iter().cloned().map(|(_, s, _)| s), limit, reverse);
                    if lines.len() > limit {
                        t.push_str("...\n");
                    }
                    tx.send(t).await.unwrap();
                    dirty = false;
                }
            }
            r = reader.read_line(&mut buf) => {
                if let Ok(size) = r {
                    if size == 0 {
                        break;
                    }
                } else {
                    eprintln!("{r:?}");
                    break;
                }
                num += 1;
                let f = if numeric {
                    parser::numeric(&buf)
                } else if human_numeric {
                    parser::human_numeric(&buf)
                } else {
                    0.0
                };
                lines.insert((FloatOrd(f), buf.clone(), num));
                buf.clear();
                dirty = true;
            }
        }
    }
    drop(tx);

    handle.await?;
    let it = lines.iter().cloned().map(|(_, s, _)| s);
    let result = if reverse {
        it.rev().collect::<String>()
    } else {
        it.collect::<String>()
    };
    let mut stdout = tokio::io::stdout();
    stdout.write_all(&result.as_bytes()).await?;
    stdout.flush().await?;
    Ok(())
}
