from pydantic import BaseSettings, PostgresDsn

class Settings(BaseSettings):
    secret_key: str
    cockroachdb_dsn: PostgresDsn
    mail_host: str
    mail_port: int
    mail_username: str
    mail_password: str
    from_email: str
    from_name: str
    email_verification_expiry_time: int
    host: str


settings = Settings().dict()