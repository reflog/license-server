# Simple License Server

## Install

`cargo install license-server`

## Usage

### CLI

```
license-server 0.1.0
Simple license server

USAGE:
    license-server.exe <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    generate    Generate a license
    help        Prints this message or the help of the given subcommand(s)
    serve       Start the license server
    validate    Validate a license
```
1. Set HMAC_SECRET env variable to your signing secret string or pass it via `-s` parameter
2. Generate a license: `license-server generate -f 2000-1-1 -u 2030-1-1 -M K=V`
3. Validate a license `license-server validate eyJsaWNlbnNlIjp7ImlkIjoiYmYwODAxNDctMTUyYS00M2M4LTg1OTMtMjI0OTA4ZTE1MzgwIiwibWV0YSI6eyJLIjoiViJ9LCJ2YWxpZF9mcm9tIjoiMjAwMC0wMS0wMSIsInZhbGlkX3VudGlsIjoiMjAzMC
   0wMS0wMSJ9LCJzaWduYXR1cmUiOiJkMzFmOTM3OWM0OTZmZGM4NDMwZmIwNmZiYmY0ZTcwN2I1NGMwZGE4OTM5MjBlOGY1MDU4YmE1ODJmM2E5MDQzIn0=`
4. Set LICENSE_API_KEY env variable to some secret string that will be used to secure `generate` endpoint or pass it via `-k` parameter
5. Start a server `license-server serve -p 3000`

### API

#### Generate a license POST `/generate`

Input format:
```json
{
  "valid_from": "2000-1-1",
  "valid_until": "2010-1-2",
  "meta": {
    "additional_data": "test",
    "user": "test user"
  }
}
```

Output format:
```json
{"result":"eyJsaWNlbnNlIjp7ImlkIjoiYmYwODAxNDctMTUyYS00M2M4LTg1OTMtMjI0OTA4ZTE1MzgwIiwibWV0YSI6eyJLIjoiViJ9LCJ2YWxpZF9mcm9tIjoiMjAwMC0wMS0wMSIsInZhbGlkX3VudGlsIjoiMjAzMC0wMS0wMSJ9LCJzaWduYXR1cmUiOiJkMzFmOTM3OWM0OTZmZGM4NDMwZmIwNmZiYmY0ZTcwN2I1NGMwZGE4OTM5MjBlOGY1MDU4YmE1ODJmM2E5MDQzIn0="}
```

#### Validate a license POST `/validate`

Input format:

```json
{
  "license": {
    "id": "123-123123-123",
    "valid_from": "2000-1-1",
    "valid_until": "2010-1-2",
    "meta": {
      "additional_data": "test",
      "user": "test user"
    }
  },
  "signature": ".........."
}
```

Results in 200 for valid license or 400 for invalid
