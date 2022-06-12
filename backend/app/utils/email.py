from fastapi import (
    BackgroundTasks,
)
from fastapi_mail import FastMail, MessageSchema, ConnectionConfig
from pydantic import EmailStr, BaseModel
from typing import List
from config import settings


class EmailSchema(BaseModel):
    email: List[EmailStr]
    subject: str
    body: str


conf = ConnectionConfig(
    MAIL_USERNAME=settings["mail_username"],
    MAIL_PASSWORD=settings["mail_password"],
    MAIL_FROM=settings["from_email"],
    MAIL_PORT=settings["mail_port"],
    MAIL_SERVER=settings["mail_host"],
    MAIL_FROM_NAME=settings["from_name"],
    MAIL_TLS=True,
    MAIL_SSL=False,
    USE_CREDENTIALS=True,
    VALIDATE_CERTS=True,
)


async def send_email(background_tasks: BackgroundTasks, email: EmailSchema):
    message = MessageSchema(
        subject=email.subject,
        recipients=email.dict().get("email"),
        body=email.body,
    )

    fm = FastMail(conf)

    background_tasks.add_task(fm.send_message, message)
    # await fm.send_message(message)
