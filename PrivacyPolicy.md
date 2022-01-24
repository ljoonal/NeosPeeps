# Privacy policy

NeosPeeps contacts the Neos API, so by using this application you're going to also be subject to [their privacy policy](https://wiki.neos.com/Neos_Wiki:Privacy_policy) as well.

## Stored configuration files

The application stores your Neos session details as well as other configuration options locally on your PC. The data is stored in plain text so you can inspect it, and isn't sent elsewhere (with the exception of using the Neos session details to authenticate to the Neos API). Your actual password, neither the totp code are never stored outside of the running application's memory.

## Cached data

The application stores requested images in the system's temporary folder to avoid re-requesting large files over and over again.

## Update checks

The application can optionally check for applications.
This either happens when the user manually presses the check for updates button, or if the auto update checking feature is enabled (disabled by default for your privacy).

The update checks contact the git host API to check retrieve the latest release.
Logs of such requests may be kept on the server side.
These logs can contain the request path, the user agent as well as timing and network related information.
The information is stored in order to keep providing the service in a secure manner and to prevent malicious use like (D)DOS attacks for example.
