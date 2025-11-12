# Q8-Caster Keycloak Integration Guide

This guide walks you through setting up Keycloak authentication for Q8-Caster.

## Prerequisites

- A running Keycloak instance
- Admin access to Keycloak
- Q8-Caster installed and configured

## Step 1: Create a Keycloak Client

1. Log in to your Keycloak Admin Console
2. Select your realm (or create a new one)
3. Navigate to **Clients** → **Create**
4. Configure the client:
   ```
   Client ID: q8-caster
   Client Protocol: openid-connect
   Root URL: http://localhost:8420
   ```

5. In the client settings:
   - **Access Type**: public (or confidential if you want client secrets)
   - **Standard Flow Enabled**: ON
   - **Direct Access Grants Enabled**: OFF
   - **Valid Redirect URIs**: `http://localhost:8420/auth/callback`
   - **Web Origins**: `http://localhost:8420`

6. If using confidential access type, note the client secret from the **Credentials** tab

## Step 2: Configure Q8-Caster

1. Copy the example configuration:
   ```bash
   cp config.toml.example config.toml
   ```

2. Edit `config.toml` with your Keycloak settings:
   ```toml
   [keycloak]
   realm = "your-realm"
   auth_server_url = "http://your-keycloak:8080"
   client_id = "q8-caster"
   # client_secret = "your-secret"  # Only if using confidential client
   redirect_uri = "http://localhost:8420/auth/callback"
   scope = ["openid", "profile", "email"]
   ```

## Step 3: Generate Encryption Key

Q8-Caster uses AES-256-GCM encryption for storing secrets. Generate a key:

```bash
openssl rand -base64 32
```

Set the key as an environment variable:
```bash
export Q8_CASTER_ENCRYPTION_KEY="your-generated-key"
```

Or add it to the systemd service file.

## Step 4: Create Secrets Directory

```bash
sudo mkdir -p /etc/q8-caster/secrets
sudo chown $USER:$USER /etc/q8-caster/secrets
chmod 700 /etc/q8-caster/secrets
```

## Step 5: Configure Roles (Optional)

In Keycloak, you can create roles to control access:

1. Navigate to **Roles** → **Add Role**
2. Create roles like:
   - `q8-caster-admin` - Full access
   - `q8-caster-viewer` - Read-only access
   - `q8-caster-operator` - Can cast content

3. Assign roles to users in **Users** → **Role Mappings**

## Step 6: Test Authentication

1. Start Q8-Caster:
   ```bash
   ./scripts/manage.sh run
   ```

2. Open http://localhost:8420 in your browser
3. Click "Login with Keycloak"
4. Complete the Keycloak login
5. You should be redirected back to the dashboard, authenticated

## API Authentication

Once authenticated, the dashboard stores the JWT token in localStorage. For API calls:

```bash
# Get token from browser console: localStorage.getItem('q8_caster_token')
TOKEN="your-jwt-token"

# Make authenticated API calls
curl -H "Authorization: Bearer $TOKEN" http://localhost:8420/api/displays
```

## Secrets Management

Q8-Caster can securely store various secrets:

### Add API Keys
```bash
curl -X POST http://localhost:8420/api/secrets/api-keys \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "chromecast-api",
    "key": "your-api-key"
  }'
```

### Add RTSP Credentials
```bash
curl -X POST http://localhost:8420/api/secrets/rtsp-credentials \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "camera_id": "front-door",
    "username": "admin",
    "password": "secure-password"
  }'
```

## Security Notes

1. **Encryption Key**: Never commit the encryption key to version control
2. **HTTPS**: Use HTTPS in production to protect tokens in transit
3. **Token Expiry**: Tokens expire based on Keycloak settings
4. **Secrets Storage**: All secrets are encrypted at rest using AES-256-GCM

## Troubleshooting

### "Invalid state parameter" error
- Ensure cookies are enabled
- Check that redirect URIs match exactly

### "Unauthorized" errors
- Verify token hasn't expired
- Check Keycloak logs for authentication issues
- Ensure client configuration matches

### Cannot discover OpenID configuration
- Verify Keycloak is accessible from Q8-Caster
- Check the auth_server_url is correct
- Ensure realm name is correct

## Advanced Configuration

### Using Groups for Authorization
Add group mapper in Keycloak client:
1. Go to client → **Mappers** → **Create**
2. Name: `groups`
3. Mapper Type: `Group Membership`
4. Token Claim Name: `groups`
5. Add to ID token: ON
6. Add to access token: ON

Q8-Caster will automatically read groups from the token.

### Custom Token Lifetimes
In Keycloak realm settings:
- Access Token Lifespan: 5 minutes (recommended)
- Client Session Idle: 30 minutes
- Client Session Max: 12 hours