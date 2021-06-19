# CONTRIBUTING 

We are excited that you are looking to contribute to memri!
The current document is not meant as a comprehensive contribution guide,
but some basic guidelines to keep us aligned:

* It might be a good idea to reach out to us on our
[Discord](https://discord.com/invite/BcRfajJk4k).
You can see which issues are/aren't being worked on by other people
in the community at the moment, and you can ask preliminary
feedback on a feature you're thinking to implement.

* We use Continuous Integration (CI) in the project.
Your Merge Requests will likely be checked by our system,
you have to make sure your contribution passes all tests.
More on those tests below as well, so that you could also understand and check them locally.

* We use [rustfmt](https://github.com/rust-lang/rustfmt) in the project.
The settings are the default for Rust (100 characters line length, etc)

* We use [clippy](https://github.com/rust-lang/rust-clippy) to ensure code quality in the project.
You can run it locally by `cargo clippy --all-targets --all-features -- -D warnings`.

* We make security audit of the libraries for each commit being made,
[cargo audit](https://github.com/RustSec/cargo-audit).

* We ask to [write test](https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html)
for all non-straightforward code in the project codebase.
We might be OK with Merge Requests without tests for declarative code like initialization
and default values, but anything involving loops/conditions is generally expected to be tested.

* We do code reviews. All Merge Requests will be reviewed by somebody in our team.
You can jump in and provide your own feedback for other Merge Requests if you want as well.

* Aim to have one MR per problem, and use branches enable that.
If you are solving a new problem before the current MR is closed, create a new MR which does not
include the changes it the first MR. If you get a request for changes for a MR, don’t submit
a new MR but proceed within the existing MR.

    Only create MR’s that change functionality. Don’t combine them with style changes, like whitespaces, symbol naming, etc.


We'll be happy to see your contributions on our GitLab server and any feedback
on our [Discord](https://discord.com/invite/BcRfajJk4k)!
