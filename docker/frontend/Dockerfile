FROM node:18

WORKDIR /usr/src/app

COPY ui/package*.json ./

RUN npm ci --omit=dev

COPY ui/. .

EXPOSE 3000

CMD [ "node", "server.js" ]