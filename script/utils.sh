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

json_attr() {
	local attr="$1"
	local input="$2"

	python -c "import json; import sys; obj = json.loads('''$input'''); print(obj['$attr'])"
}
