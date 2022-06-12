import base64
from datetime import datetime, timedelta
from typing import Optional
from jwt import *
import json

from fastapi import Request
from passlib.hash import argon2
from config import settings

SECRET_KEY = settings['secret_key']
ALGORITHM = "HS256"


def verify_hash(password, savedSalt, hashed_password):
    savedSalt = savedSalt.encode("utf-8")
    savedSalt = base64.b64decode(savedSalt)
    return argon2.using(salt=savedSalt).verify(
        password,
        hashed_password
    )


def create_access_token(data: dict, expires_delta: Optional[timedelta] = None):
    to_encode = data.copy()
    if expires_delta:
        expire = datetime.utcnow() + expires_delta
    else:
        expire = datetime.utcnow() + timedelta(minutes=15)

    to_encode.update({"exp": expire.isoformat()})
    encoded_jwt = encodeJWT(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    return encoded_jwt


def get_current_user_email(token):
    decoded = decodeJWT(token, SECRET_KEY)
    # email: str = payload["sub"]
    user_email = json.loads(decoded["payload"])["sub"]
    return user_email


def verify_user(request: Request):
    try:
        jwt = request.cookies.get('jwt')
        xsrf_token = request.headers['X-XSRF-Token']
        r= decodeJWT(jwt, SECRET_KEY)
        p = json.loads(r['payload'])
        v = r['verified']
        if not v:
            return False
        if xsrf_token == p['xsrf_token']:
            return p['user']['email']
        else:
            return False
    except:
        return False
