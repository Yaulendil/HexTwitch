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

After Cargo compiles the plugin, its Binary should be in `target/release/`, and should be named something like `libhextwitchr.so` or, on Windows, `libhextwitchr`. Move this File into the `addons` Directory in your HexChat config Directory; On Linux this should be at `$XDG_CONFIG_HOME/hexchat/addons/`.

If you have not patched HexChat, it will probably crash. Otherwise, you should now have Twitch features.
