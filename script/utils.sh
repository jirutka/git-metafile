# vim: set ts=4:

die() {
	# bold red
	printf '\033[1;31mERROR:\033[0m %s\n' "$1" >&2
	exit ${2:-2}
}

einfo() {
	# bold cyan
	printf '\n\033[1;36m> %s\033[0m\n' "$@"
}

crate_name() {
	cargo metadata --no-deps | sed -En 's/.*"id":"(\S+).*/\1/p'
}

crate_version() {
	cargo metadata --no-deps | sed -En 's/.*"id":"\S+ (\S+).*/\1/p'
}
