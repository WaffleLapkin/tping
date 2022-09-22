// FAQ
//
// Q: why is this so over-engineered?
// A: yes
//
// Q: why
// A: so that I can do `x b && tping`, go do some stuff in another room and
//    be notified when the build finishes
//
// Q: what's up with error handling?
// A:
// FIXME: [nicer] error handling
//
// Q: why KDL?
// A: it's cuteb

use teloxide_core::{
    requests::{Requester, RequesterExt},
    types::ChatId,
    Bot,
};
use tokio::runtime::Builder;

fn main() {
    let conf = config();

    let dst = conf.destination(std::env::args().nth(1).as_deref());
    let bot = Bot::new(conf.token()).auto_send();

    let rt = Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .expect("Couldn't create async runtime :(");

    rt.block_on(bot.send_message(dst, "ping!"))
        .expect("Couldn't send message");
}

#[derive(Debug, Default, knuffel::Decode)]
struct Config {
    #[knuffel(child, unwrap(argument))]
    token: Option<String>,
    #[knuffel(child, unwrap(argument))]
    default: Option<i64>,
    #[knuffel(child, unwrap(children), default)]
    routes: Vec<Route>,
}

#[derive(Debug, knuffel::Decode)]
struct Route {
    #[knuffel(node_name)]
    name: String,
    #[knuffel(argument)]
    dst: i64,
}

fn config() -> Config {
    let path = dirs::config_dir().unwrap().join("tping.kdl");
    let conf = match std::fs::read_to_string(&path) {
        Ok(conf) => conf,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return <_>::default(),
        err @ Err(_) => err.unwrap(),
    };

    let conf: Config = knuffel::parse(&path.to_string_lossy(), &conf).unwrap();

    conf
}

impl Config {
    fn token(&self) -> String {
        const ERR: &str = "Couldn't find token for the telegram bot. Either add it to the config in the form of `token \"<...>\"`, or provide it in `TPING_TOKEN` or `TELOXIDE_TOKEN` env variables";
        self.token
            .clone()
            .or_else(|| std::env::var("TPING_TOKEN").ok())
            .or_else(|| std::env::var("TELOXIDE_TOKEN").ok())
            .expect(ERR)
    }

    fn destination(&self, name: Option<&str>) -> ChatId {
        let id = match name {
            None => self.default.unwrap(),
            Some(name) => self.routes.iter().find(|r| r.name == name).unwrap().dst,
        };

        ChatId(id)
    }
}
