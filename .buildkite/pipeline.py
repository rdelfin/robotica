from utils import run, get_package_step, to_json

# By default, do nothing.
steps = []

# Query the Bazel workspace for a list of all packages (libraries, binaries, etc.).
all_packages = run(["bazel", "query", "//..."])

# Using both lists, figure out which packages need to be built.
for pkg in all_packages:
    package_step = get_package_step(pkg)

# Emit the pipeline as JSON to be uploaded to Buildkite.
print(to_json({"steps": steps}, 4))
