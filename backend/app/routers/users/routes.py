import re
from typing import List
from uuid import uuid4

from fastapi import APIRouter, HTTPException, Response, BackgroundTasks, Header
from fastapi.responses import ORJSONResponse

from config import settings
import routers.users.schemas as schemas
import routers.users.crud as crud
import app.security as security
from utils.token import get_token, verify_token
from utils.email import send_email, EmailSchema

router = APIRouter(prefix="/api")


@router.post("/register", response_model=schemas.User, response_class=ORJSONResponse)
async def create_user(user: schemas.UserCreate, background_task: BackgroundTasks):
    if user.password != user.confirm_password and len(user.password) < 16:
        raise HTTPException(status_code=409, detail="Invalid password")
    regex = r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b"
    db_user = await crud.get_user_by_email(email=user.email)
    if re.fullmatch(regex, user.email) == None:
        raise HTTPException(status_code=409, detail="Invalid email")
    if db_user:
        raise HTTPException(status_code=409, detail="Email already registered")
    db_user = await crud.get_user_by_username(username=user.username)
    if db_user:
        raise HTTPException(status_code=409, detail="Username unavailable")
    token = await get_token(user.email)
    body = f"""Hi {user.username},

Thank you for registering at Kunjika.
Your email confirmation link is https://{settings['host']}/confirm-email/{token.decode('utf-8')}.
This email will expire in one day.

Thanks,
Shiv
"""
    email = EmailSchema(
        email=[user.email], subject="Registration at Kunjika", body=body
    )
    await send_email(email=email, background_tasks=background_task)
    return await crud.create_user(user=user)


@router.get("/users/", response_model=List[schemas.User], response_class=ORJSONResponse)
async def read_users(skip: int = 0, limit: int = 100):
    users = crud.get_users(skip=skip, limit=limit)
    return users


@router.get(
    "/users/{user_id}", response_model=schemas.User, response_class=ORJSONResponse
)
async def read_user(user_id: int):
    db_user = crud.get_user(user_id=user_id)
    if db_user is None:
        raise HTTPException(status_code=404, detail="User not found")
    return db_user


@router.post("/login/", response_class=ORJSONResponse)
async def login(user: schemas.Login, response: Response):
    db_user = await crud.get_user_by_email(email=user.email)
    # we save jwt to cookie to prevent XSS and supply xsrf_token in this to prevent
    # CSRF attacks. This should come back in X-XSRF-Token
    xsrf_token = str(uuid4())
    if db_user:
        if security.verify_hash(user.password, db_user.salt, db_user.password_hash):
            access_token = security.create_access_token(
                data={
                    "sub": db_user.email,
                    "user": {
                        "username": db_user.username,
                        "email": db_user.email,
                        "id": str(db_user.id),
                    },
                    "xsrf_token": xsrf_token,
                }
            )
            # the expires is 2^31- 1 which is max 32 bit signed integer
            # the cookie must be http only so that XSS is disabled
            response.set_cookie(key="jwt", value=access_token, expires=2147483647, secure=True)
            return {"success": True}
        else:
            raise HTTPException(status_code=401, detail="Unauthorized")
    else:
        raise HTTPException(status_code=401, detail="Unauthorized")


@router.post(
    "/check-username-availability",
    response_model=schemas.Availability,
    response_class=ORJSONResponse,
)
async def check_username_availability(username: schemas.Username):

    if await crud.get_user_by_username(username=username.username):
        return {"available": False}
    else:
        return {"available": True}


@router.get("/confirm-email/{token}", response_class=ORJSONResponse)
async def confirm_email(token: str):
    try:
        email = await verify_token(token)
        await crud.verify_email(email)
        return {"success": True}
    except Exception as e:
        print(e)
        raise HTTPException(status_code=401, detail="Invalid or expired token.")

@router.post("/logout")
async def logout( response: Response, x_token: list[str] | None = Header(default=None), response_class=ORJSONResponse):
    response.set_cookie(key="jwt", value='', max_age=0, secure=True)
    return {"success": True}
