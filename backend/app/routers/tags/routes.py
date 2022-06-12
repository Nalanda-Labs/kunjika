from fastapi import APIRouter, HTTPException, Depends
from fastapi.responses import ORJSONResponse

import security
from app.routers.tags import schemas
from app.routers.tags.crud import get_tags_like


router = APIRouter(prefix="/api")


@router.post("/get-tags/")
async def get_tags(tag: schemas.TagBase, response_class=ORJSONResponse, email: str = Depends(security.verify_user)):
    if not email:
        raise HTTPException(status_code=401, detail="Unauthorized")

    return await get_tags_like(tag.tag)
