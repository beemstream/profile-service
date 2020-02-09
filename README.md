## Auth service
### Rest Endpoints
#### Login
```
POST /login
```
Request login 
```
{
  username: string,
  password: string,
}
```
Response login
```
{
  oauth-token: {
    token: string,
    refresh_token: string,
    refresh_interval: string,
  }
}
```
#### Registration
```
POST /register
```
Request register 
```
{
  username: string,
  password: string,
  email: string
}
```
Response register
```
{
  status: string
}
```

#### Refresh token
```
POST /refresh-token
```
Request refresh 
```
{
  token: string,
}
```
Response refresh
```
{
  token: string,
  refresh_token: string
}
`
