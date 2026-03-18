FROM ruby:3.3-slim
RUN apt-get update && apt-get install -y --no-install-recommends build-essential && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Gemfile .
RUN bundle install
COPY send_log.rb .
CMD ["ruby", "send_log.rb"]
