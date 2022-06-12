import base64
import os

from jwt import *
from passlib.hash import argon2

from models import Users
import routers.users.schemas as schemas


async def get_user(user_id: int):
    return await Users.filter(Users.id == user_id).first()


async def get_user_by_email(email: str):
    return await Users.filter(email=email).first()


async def get_users(skip: int = 0, limit: int = 100):
    return await Users.offset(skip).limit(limit).all()


async def create_user(user: schemas.UserCreate):
    salt = os.urandom(32)
    key = argon2.using(salt=salt).hash(
        user.password.encode("utf-8")
    )
    # Bytes encoded to Base64 but still in byte format
    encodedSalt = base64.b64encode(salt)

    db_user = Users(
        username=user.username,
        email=user.email,
        password_hash=key,
        salt=encodedSalt.decode("utf-8"),
        username_lower = user.username.lower()
    )
    await db_user.save()
    return db_user

async def get_user_by_username(username: str):
    return await Users.filter(username=username).first()

async def verify_email(email: str):
    return await Users.filter(email=email).first().update(email_verified=True)