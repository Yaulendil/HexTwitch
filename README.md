[![License: GPLv3](img/gplv3-127x51.png)](https://opensource.org/licenses/GPL-3.0)

# HexTwitch: Rust Edition

***NOTE: This Plugin can ONLY be used with a modified version of HexChat. The Patch File is included in this Repository at `./hex.patch`.***

This is a Plugin for [HexChat](https://github.com/hexchat/hexchat/) which aims to integrate some of the more advanced features of the [Twitch](https://twitch.tv) Chat IRC bridge; Specifically, user Badges and channel events such as Subscriptions. This information is supplied via IRCv3 Tags.

There are various IRC clients built specifically for Twitch, such as [Chatty](https://github.com/chatty/chatty), but none seem to compare to HexChat in terms of performance. This is, therefore, an attempt to rework HexChat into a Twitch chat client, without altering its behavior on other IRC Networks.

One problem, however, stands in the way: The HexChat Plugin interface does not provide Callbacks with the IRC Tags. Therefore we must modify HexChat to add this functionality, using the Patch File mentioned above (This could be submitted as a Pull Request, but currently, my C sucks and I would rather not waste their time).
