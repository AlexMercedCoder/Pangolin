# Environment Variables

**This page has moved to [../environment-variables.md](../environment-variables.md).**

Two hand-maintained environment-variable references existed side by side and
both had drifted from the code (B43). Keeping one of them was not a fix - the
problem was that either could drift silently.

The surviving reference is checked against the source in CI by
`pangolin/scripts/check_env_var_docs.sh`, which fails the build if it documents
a `PANGOLIN_*` variable nothing reads, or omits one something does. This page is
a redirect so existing links keep working.
