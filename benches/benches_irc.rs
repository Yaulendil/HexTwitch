#![feature(test)]

extern crate test;

use hextwitchr::irc::*;
use test::Bencher;

const MSG_WITHOUT_TAGS: &str =
    r":asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv WHISPER thyself :asdf";
const SAMPLES: &[&str] = &[
    MSG_WITHOUT_TAGS,
    r"@badges=bits/100;display-name=AsdfQwert;emotes= :asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv PRIVMSG #zxcv arg2 :this is a message",
    r"@badges=subscriber/24,badge-info=subscriber/25,bits/1;emote-sets=0,2652,15749,19194,230961,320370,1228598;user-type= :tmi.twitch.tv USERSTATE #asdfqwert",
    r"@login=somejerk;room-id=;target-msg-id=15604c60-4d3b-8c1c-8e7a-c9ec2fb6c0cf;tmi-sent-ts=-6745368778951 :tmi.twitch.tv CLEARMSG #coolchannel :get a real job noob",
    r"@room-id=;target-user-id=8675309;tmi-sent-ts=1582958744397 :tmi.twitch.tv CLEARCHAT #coolchannel :somejerk",
    r"@badges=;color=#DABEEF;display-name=Asdf\sQwert;emotes=;message-id=2;thread-id=1337-9001;turbo=0;user-id=123456789;user-type= :asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv WHISPER thyself :asdf",
];

/// Benchmark performance of `&str`s being parsed into `Message`s.
#[bench]
fn bench_samples_0tags(b: &mut Bencher) {
    for init in SAMPLES {
        b.iter(|| {
            let _msg: Message = init.parse().expect("Parse Failed");
        });
    }
}

/// Benchmark performance of `&str`s being parsed into `Message`s and having
///     IRC Tag values extracted.
#[bench]
fn bench_samples_3tags(b: &mut Bencher) {
    for init in SAMPLES {
        b.iter(|| {
            let msg: Message = init.parse().expect("Parse Failed");
            msg.get_tag("bits");
            msg.get_tag("badges");
            msg.get_tag("badge-info");
        });
    }
}

/// Benchmark performance of `&str`s being parsed into `Message`s and back.
#[bench]
fn bench_samples_pingpong(b: &mut Bencher) {
    for init in SAMPLES {
        b.iter(|| init.parse::<Message>().expect("Parse Failed").to_string());
    }
}

/// Benchmark performance of a `Message` having its tags set by method.
#[bench]
fn bench_samples_set_tag(b: &mut Bencher) {
    for init in SAMPLES {
        b.iter(|| {
            let mut msg: Message = init.parse().expect("Parse Failed");
            msg.set_tag("bits", "asdf").ok();
            msg.set_tag("badges", "asdf").ok();
            msg.set_tag("badge-info", "asdf").ok();
        });
    }
}
