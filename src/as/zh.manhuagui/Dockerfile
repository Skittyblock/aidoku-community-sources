FROM node:16.17.0

WORKDIR /usr/app

COPY ./ ./
COPY src src
COPY res res

RUN apt-get update
RUN apt-get install zip -y
RUN npm install
RUN npm run build

CMD ["npm", "run", "serve"]