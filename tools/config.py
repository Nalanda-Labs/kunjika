from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    database_url: str

    class Config:
        # adjust this to match the env file
        env_file = '../backend/.env'
        env_file_encoding = 'utf-8'

settings = Settings(_env_file_encoding='utf-8')

