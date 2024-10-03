# Quickstart

Installing Sealed Infrastructure is easy.

```bash
curl -sSfL https://sealedinfra.io/get | sh
```

### CLI Building a run command


Building a run command is easy.

```bash
si --settings ./config/config.yaml docker --repo git@bitbucket.org:financialpayments/tupay.git --branch "origin/eol/upgrade" build  
# Or from the source directory:
cargo run -- --settings ./config/config.yaml docker --repo git@bitbucket.org:financialpayments/tupay.git --branch "origin/eol/upgrade" build
```

This `build` command gives us a run command we can use to run the container.

```bash
docker run -v /etc:/etc:ro -v type=tmpfs,tmpfs-size=10000000000,tmpfs-mode=0777:/app,destination=/app:ro -e HOME=/app tupay:latest
```

### CLI Building a run command

