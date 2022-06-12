from pydantic import BaseModel


class TagBase(BaseModel):
    tag: str
