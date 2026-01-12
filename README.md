# tcurse

## Release

Update the version in `Cargo.toml`

```
git tag {version}
git push --tags
```

`dist` automatically takes care of building releases.

### Notes on dist

- `dist init` and follow the CLI to setup a project for publication.
- `publish-jobs = ["npm"]` was required to publish to npm. For some reason `dist init` did not do this. Think it's a bug.
- You need an NPM account and auth token wired up to Github repo secrets to publish to NPM via `dist`.
- To delete a tag locally: `git tag -d {tag}`
- To delete a tag remotely: `git push --delete origin {tag}`
