#!/bin/bash
# Script to automatically download and patch HexChat source code. Then,
# optionally, to also compile and/or install the patched client.

# ENVIRONMENT #
DO_BUILD=${DO_BUILD:-false} # Build HexChat after patching it. Default false.
DO_INSTALL=${DO_INSTALL:-false} # Install HexChat after building it. Default false. Implies $DO_BUILD.
$DO_INSTALL && DO_BUILD=true # Need to build in order to install.

# CONSTANTS #
REPO="https://github.com/hexchat/hexchat.git"
PATH_BUILD=hexchat/
PATH_PATCH=../hex.patch

URL_DL_GIT="https://git-scm.com/downloads"
URL_DL_PATCH="https://savannah.gnu.org/projects/patch"
URL_DL_MESON="https://mesonbuild.com/Getting-meson.html"
URL_DL_NINJA="https://ninja-build.org/"
URL_DOCS_HEX="https://hexchat.readthedocs.io/en/latest/building.html"


# if ! [[ -e "$PATH_BUILD/$PATH_PATCH" ]]; then
# 	name=$(basename "$PATH_PATCH")
# 	echo "Patch file at $name not found."
# 	exit 1
# fi


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
	fi
}

# Terminate the script if dependencies are not met. This is in a separate
# function so that all of the missing tools can be shown at the same time.
required_now() { $missing_deps && exit 1; }


requirements
req git "Git needs to be installed to download HexChat source code. \
Download it from here:
	$URL_DL_GIT"

req patch "GNU Patch needs to be installed to modify the HexChat source code. \
Download it from here:
	$URL_DL_PATCH"

$DO_BUILD && req meson "Meson needs to be installed in order to build HexChat. \
Instructions are available here:
	$URL_DL_MESON"

$DO_BUILD && req ninja "The Ninja backend for Meson needs to be installed in \
order to build and install HexChat. \
It is typically included with Meson, but is also available here:
	$URL_DL_NINJA"
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

$DO_BUILD || finish "Source code patched. You can now build and install \
HexChat normally from the $PATH_BUILD directory. For details on how to do \
this, refer to this page:
	$URL_DOCS_HEX"


# BUILD #
echo "Source code patched. Beginning compilation."
pushd "$PATH_BUILD"
meson build || error 4 "Failed to build HexChat with Meson."
ninja -C build || error 5 "Failed to build HexChat with Ninja."
$DO_INSTALL || finish "HexChat has been compiled successfully."


# INSTALL #
echo "Compilation complete. Now installing."
sudo ninja -C build install || error 6 "Failed to install HexChat with Ninja."
finish "HexChat has been installed successfully."
