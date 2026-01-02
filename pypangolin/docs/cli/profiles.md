# Profile Management

The Pangolin CLI uses profiles to manage connection details for different environments (e.g., local, dev, prod) or different users. Configuration is stored in `~/.pangolin/profiles.yaml`.

## Configuration File

The default configuration file structure looks like this:

```yaml
active_profile: default
profiles:
  default:
    url: http://localhost:8080
    username: admin
    token: null
    tenant_id: null
```

## Commands

### `profiles`
Lists all available profiles defined in your configuration file. The active profile is marked with an asterisk (`*`).

**Example:**
```bash
$ pypangolin profiles
* default
  prod
```

### `use`
Switches the active profile.

**Arguments:**
- `PROFILE_NAME`: The name of the profile to activate.

**Example:**
```bash
pypangolin use prod
```

## Managing Profiles

Currently, the CLI does not have dedicated `create-profile` commands. You can manually edit `~/.pangolin/profiles.yaml` to add new profiles.

**Example `profiles.yaml` addition:**

```yaml
profiles:
  prod:
    url: https://pangolin.mycompany.com
    username: myuser
```
