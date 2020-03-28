## Auth service
### Rest Endpoints
#### Login
```
POST /login
```
Request login 
```
{
  identifier: string,
  password: string,
}
```
Response login
```
{
  token: string,
  refreshToken: string,
  refreshInterval: string,
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
  email: string
  password: string,
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
  refreshInterval: string
}
`
