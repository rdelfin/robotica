from utils import run, get_package_step, to_json

# By default, do nothing.
steps = []

# Query the Bazel workspace for a list of all packages (libraries, binaries, etc.).
all_packages = run(["bazel", "query", "//..."])
steps = [get_package_step(pkg) for pkg in all_packages]

# Emit the pipeline as JSON to be uploaded to Buildkite.
print(to_json({"steps": steps}, 4))
