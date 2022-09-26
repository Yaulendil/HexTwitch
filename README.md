[![License: GPLv3](img/gplv3-127x51.png)](https://opensource.org/licenses/GPL-3.0)

# HexTwitch: Rust Edition

***NOTE: This Plugin can ONLY be used with a modified version of HexChat. The Patch File is included in this Repository at `./hex.patch`.***

This is a Plugin for [HexChat](https://github.com/hexchat/hexchat) which aims to integrate some of the more advanced features of the [Twitch](https://twitch.tv) Chat IRC bridge; Specifically, user Badges and channel events such as Subscriptions. This information is supplied via IRCv3 Tags.

![Screenshot](img/ross.png)

There are various IRC clients built specifically for Twitch, such as [Chatty](https://github.com/chatty/chatty), but none seem to compare to HexChat in terms of performance. This is, therefore, an attempt to rework HexChat into a Twitch chat client, without altering its behavior on other IRC Networks.

One problem, however, stands in the way: The HexChat Plugin interface does not provide Callbacks with the IRC Tags. Therefore we must modify HexChat to add this functionality, using the Patch File mentioned above (This could be submitted as a Pull Request, but currently, my C sucks and I would rather not waste their time).

## Patching HexChat

With [Git](https://git-scm.com) and [GNU Patch](https://savannah.gnu.org/projects/patch) installed, the following commands should download and patch the latest HexChat source code:

```
git clone https://github.com/hexchat/hexchat.git build/hexchat
patch -p0 -d build/hexchat -i ../../hex.patch
```

When the patch is applied, you can then [build and install HexChat as normal](https://hexchat.readthedocs.io/en/latest/building.html) from its directory at `build/hexchat/`.

## Building the Plugin

You will need to have [Cargo](https://github.com/rust-lang/cargo) installed. Cargo can be installed and set up easily with [RustUp](https://rustup.rs/). When this is done, run the following command in the Directory where you unpacked this Repository:

```
cargo build --release
```

After Cargo compiles the plugin, its Binary should be in `target/release/`, and should be named something like `libhextwitch` or `libhextwitch.so`. Move this File into the `addons` Directory in your HexChat config Directory; On Linux, this should be at `$XDG_CONFIG_HOME/hexchat/addons/`.

If you have not patched HexChat, it will probably crash. Otherwise, you should now have Twitch features.


## Badge Icons

It is not currently feasible to embed images inline in HexChat without creating a custom font. In lieu of that potential far-future solution, and to keep installation as simple as possible, Twitch user badges are mapped to Unicode codepoints, as detailed in the following lists.

Badges without a codepoint specified will be rendered as `?`. A list of all unknown badge names can be viewed with the `/UNKNOWNS` command. This information should be provided as an Issue in this repository, so that they can be added to the plugin.

Note that some typefaces may render some of these characters in an Emoji style, which may clash somewhat with the rest of the interface.

### Global
- `🜨` (![staff](img/badges/staff.png)): Twitch staff.
- `⛨` (![admin](img/badges/admin.png)): Twitch administrator.
- `✓` (![partner](img/badges/partner.png)): Twitch partner.
- `a` (![ambassador](img/badges/ambassador.png)): Community "Ambassador", handpicked by Twitch.
- `Δ` (![game-developer](img/badges/game-developer.png)): This account is registered with Twitch as a game developer or publisher.
- `+` (![turbo](img/badges/turbo.png)): Twitch Turbo member.
- `±` (![premium](img/badges/premium.png)): Twitch Prime member.
- `~` (![glhf-pledge](img/badges/glhf-pledge.png)): User has taken the "GLHF Pledge".
- `g` (![glitchcon](img/badges/glitchcon2020.png)): User has attended GlitchCon.
- `c` (![twitchcon](img/badges/twitchcon2017.png)): User has attended TwitchCon.
- `w` (![overwatch-league-insider](img/badges/overwatch-league-insider_2018B.png)): Overwatch League "Insider".
- `G`: This icon represents various game-specific badges.

### Channel-specific
- `🜲` (![broadcaster](img/badges/broadcaster.png)): User is the Broadcaster, the owner of the channel.
- `🗡` (![moderator](img/badges/moderator.png)): User is a Moderator of this channel with additional powers, handpicked by the Broadcaster.
- `⚑` (![vip](img/badges/vip.png)): User is a VIP of this channel, handpicked by the Broadcaster.
- `ⲷ` (![founder](img/badges/founder.png)): User was one of the first subscribers to this channel.
- `α` (![artist-badge](img/badges/artist-badge.png)): User has contributed art or media to this channel.
- `m` (![moments](img/badges/moments-4.png),![moments](img/badges/moments-8.png),![moments](img/badges/moments-12.png),![moments](img/badges/moments-16.png),![moments](img/badges/moments-20.png)): User was active in chat for a notable event, chosen by the Broadcaster.
- `.` (![hype-train](img/badges/hype-train-1.png),![hype-train](img/badges/hype-train-2.png)): User has contributed to a Hype Train in this channel.
- `Ⓐ` (![no_audio](img/badges/no_audio.png)): User is watching the stream with no audio.
- `Ⓥ` (![no_video](img/badges/no_video.png)): User is listening to the stream with no video.

### Cheering/Bits
- `*` (![anonymous-cheerer](img/badges/anonymous-cheerer.png)): User is an anonymous cheerer.
- `❖` (![bits-leader](img/badges/bits-leader-1.png),![bits-leader](img/badges/bits-leader-2.png),![bits-leader](img/badges/bits-leader-3.png)): User is one of the top three cheerers in this channel.
- `🝔` (![bits-charity](img/badges/bits-charity.png)): User has given bits with the `#charity` tag.
- `▴` (![bits](img/badges/bits-1.png)): User has given at least 1 bit ($0.01).
- `⬧` (![bits](img/badges/bits-100.png)): User has given at least 100 bits ($1.00).
- `⬠` (![bits](img/badges/bits-1000.png)): User has given at least 1,000 bits ($10.00).
- `⬡` (![bits](img/badges/bits-5000.png)): User has given at least 5,000 bits ($50.00).
- `🟋` (![bits](img/badges/bits-10000.png)): User has given at least 10,000 bits ($100.00).
- `🟎` (![bits](img/badges/bits-100000.png)): User has given at least 100,000 bits ($1,000.00).

### Subscriptions (![subscriber](img/badges/subscriber-0.png))
- `⁘` (![sub-gift-leader](img/badges/sub-gift-leader-1.png),![sub-gift-leader](img/badges/sub-gift-leader-2.png),![sub-gift-leader](img/badges/sub-gift-leader-3.png)): User is one of the top three givers of gift subscriptions in this channel.
- `:` (![sub-gifter](img/badges/sub-gifter-1.png)): User has given gift subscriptions in this channel.
- Double-circled digits (`⓵`,`⓷`,`⓺`,`⓽`) are used to represent subscriptions below 1 year.
- Roman Numerals (`ⅰ`,`ⅱ`,`ⅲ`,`ⅳ`,`ⅴ`,`ⅵ`,`ⅶ`,`ⅷ`,`ⅸ`,`ⅹ`,`ⅺ`,`ⅻ`) are used to represent subscriptions of 1 year up to 12 years.
- Inverted circled numbers `⓭` through `⓴` are used to represent subscriptions longer than 12 years.
- `⁑`: User is a subscriber of at least 21 years. This one should not show up for quite a while.

### Channel Points Predictions
- `❶`–`❿` (![blue-1](img/badges/predictions-blue-1.png)–![blue-10](img/badges/predictions-blue-10.png)): User has bet channel points on a blue prediction, with up to 10 possible outcomes.
- `❶`/`❷` (![blue-1](img/badges/predictions-blue-1.png),![pink-2](img/badges/predictions-pink-2.png)): User has bet channel points on a blue/pink prediction, with only two possible outcomes.
- `⧲`/`⧳` (![gray-1](img/badges/predictions-gray-1.png),![gray-2](img/badges/predictions-gray-2.png)): User has bet channel points on a gray prediction, with only two possible outcomes.
- `p`: This codepoint represents an unknown Prediction badge, likely because Twitch added a new one again. If this is seen, an Issue should be opened in this repository, including the output of running the `/UNKNOWNS` command.


[Breeze Icons](https://develop.kde.org/frameworks/breeze-icons/) used for HexChat GUI menu items © KDE, licensed under the GNU LGPL 3 or later.
