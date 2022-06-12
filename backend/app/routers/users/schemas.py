from pydantic import BaseModel


class UserBase(BaseModel):
    email: str


class UserCreate(UserBase):
    username: str
    password: str
    confirm_password: str


class User(UserBase):
    id: int
    active: bool

    class Config:
        orm_mode = True


class Login(BaseModel):
    email: str
    password: str


class Availability(BaseModel):
    available: bool


class Username(BaseModel):
    username: str
