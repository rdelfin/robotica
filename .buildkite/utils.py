import json, os, subprocess, sys

CI_COMPOSE_PLUGIN = {
    "docker-compose#v5.5.0": {
        "config": ".buildkite/docker-compose.yml",
        "run": "ci",
        "mount-buildkite-agent": True,
    }
}

# Returns a Buildkite `command` step that builds, tests, and annotates a Bazel package.
def get_package_step(package):
    return command_step(
        "bazel",
        f"Build and test //{package}/...",
        [
            f"bazel test //{package}/... --test_output=errors --verbose_failures --curses=no --color=no",
            f"bazel build //{package}/... --build_event_json_file=bazel-events.json --verbose_failures --curses=no --color=no",
        ],
        [
            CI_COMPOSE_PLUGIN,
            { "bazel-annotate#v0.1.0": { "bep_file": f"bazel-events.json"} },
        ],
    )

# Returns a Buildkite `command` step (as a Python dictionary.
def command_step(emoji, label, commands=[], plugins=[]):
    step = {"label": f":{emoji}: {label}", "commands": commands}
    if plugins:  # Simplified conditional check
        step["plugins"] = plugins
    return step

# Runs an OS command, returning stdout as a list of lines.
def run(command):
    print(f"+ {' '.join(command)}", file=sys.stderr)
    result = subprocess.run(command, capture_output=True, text=True)
    if result.stderr:
        print(result.stderr, file=sys.stderr)
    if result.returncode != 0:
        print(f"Command failed with exit code {result.returncode}", file=sys.stderr)
        print(f"stdout:\n{result.stdout}", file=sys.stderr)
        sys.exit(result.returncode)
    return result.stdout.strip().splitlines()

# Converts a list of file paths into a list of directories, omitting those that aren't.
def filter_dirs(paths):
    return [p for p in paths if os.path.isdir(p)]

# Converts a list of Bazel targets into a unique list of top-level paths. For example,
# turns this:
#   //app:main
#   //app:test_main
#   //library:hello
#   //library:test_hello
#
# into this:
#   app
#   library
def get_paths(targets, exclude=None):
    groups = {directory.lstrip("/") for directory, _, _ in (target.rpartition(":") for target in targets)}
    if exclude:
        groups.discard(exclude)
    return list(groups)


# Converts a Python dictionary into a JSON string, with optional pretty-printing.
def to_json(data, indent=None):
    return json.dumps(data, indent=indent)
