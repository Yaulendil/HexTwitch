#!/bin/bash
# Script to automatically download and patch HexChat source code, and then to
# also compile and install the patched client.

# ENVIRONMENT #
NO_BUILD=${NO_BUILD:-false} # Do not build HexChat after patching it. Default false. Implies $NO_INSTALL.
NO_INSTALL=${NO_INSTALL:-false} # Do not install HexChat after building it. Default false.
$NO_BUILD && NO_INSTALL=true # Need to build in order to install.

# CONSTANTS #
REPO="https://github.com/hexchat/hexchat.git"
FILE_PATCH=hex.patch
PATH_BUILD=hexchat/
PATH_PATCH=../$FILE_PATCH

NEED_GIT="Git needs to be installed to download HexChat source code. \
Download it from here:
	https://git-scm.com/downloads"
NEED_PATCH="GNU Patch needs to be installed to modify the HexChat source code. \
Download it from here:
	https://savannah.gnu.org/projects/patch"
NEED_MESON="Meson needs to be installed in order to build HexChat. \
Instructions are available here:
	https://mesonbuild.com/Getting-meson.html"
NEED_NINJA="The Ninja backend for Meson needs to be installed in order to \
build and install HexChat. \
It is typically included with Meson, but is also available here:
	https://ninja-build.org/"
URL_DOCS_HEX="https://hexchat.readthedocs.io/en/latest/building.html"


if ! [[ -e "$FILE_PATCH" ]]; then
	echo "Patch file at '$FILE_PATCH' not found."
	exit 1
fi


# SETUP #
missing_deps=false

# Specify that a list of requirements will follow. Reset the missing status so
# previous failed requirements will not be considered.
requirements() { missing_deps=false; }

# Specify that a command is needed in order to proceed.
req() {
	if ! command -v "$1" 1>/dev/null 2>/dev/null; then
		missing_deps=true

		if [[ -n "$2" ]]; then
			echo "$2"
		fi

		return 1
	else
		return 0
	fi
}

# Terminate the script if dependencies are not met. This is in a separate
# function so that all of the missing tools can be shown at the same time.
required_now() { $missing_deps && exit 1; }


requirements
req git "$NEED_GIT" # Need Git.
req patch "$NEED_PATCH" # Need GNU Patch.
$NO_BUILD || req meson "$NEED_MESON" # Need Meson (if building).
$NO_BUILD || req ninja "$NEED_NINJA" # Need Ninja (if building).
required_now


error() {
	if [[ -n "$2" ]]; then
		echo "$2"
	fi
	popd 1>/dev/null 2>/dev/null
	exit $1
}
finish() { error 0 "$@"; }


# CLONE+PATCH #
git clone "$REPO" "$PATH_BUILD" || error 2 "Failed to clone HexChat."
echo "Downloaded HexChat source code. Applying patch."
patch -p0 -d "$PATH_BUILD" -i "$PATH_PATCH" || error 3 "Could not patch the \
source code. It is likely that HexChat has been updated since this script was \
written."

$NO_BUILD && finish "Source code patched. You can now build and install \
HexChat normally from the '$PATH_BUILD' directory. For details on how to do \
this, refer to this page:
	$URL_DOCS_HEX"


# BUILD #
echo "Source code patched. Beginning compilation."
pushd "$PATH_BUILD"
meson build || error 4 "Failed to build HexChat with Meson."
ninja -C build || error 5 "Failed to build HexChat with Ninja."
$NO_INSTALL && finish "HexChat has been compiled successfully."


# INSTALL #
echo "Compilation complete. Now installing."
req sudo "Sudo is not available. Will attempt to proceed without it, but the \
installation is likely to fail." \
	&& sudo=sudo \
	|| sudo=
$sudo ninja -C build install || error 6 "Failed to install HexChat with Ninja."
finish "HexChat has been installed successfully."
