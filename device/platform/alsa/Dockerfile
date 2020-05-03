FROM debian:latest
RUN apt-get update \
    && apt-get install -y \
        libasound2-dev \
        pulseaudio \
        pulseaudio-utils \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
COPY pulse-client.conf /etc/pulse/client.conf

ENV UNAME user

# Set up the user
RUN export UNAME=$UNAME UID=1000 GID=1000 && \
    mkdir -p "/home/${UNAME}" && \
    echo "${UNAME}:x:${UID}:${GID}:${UNAME} User,,,:/home/${UNAME}:/bin/bash" >> /etc/passwd && \
    echo "${UNAME}:x:${UID}:" >> /etc/group && \
    mkdir -p /etc/sudoers.d && \
    echo "${UNAME} ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/${UNAME} && \
    chmod 0440 /etc/sudoers.d/${UNAME} && \
    chown ${UID}:${GID} -R /home/${UNAME} && \
    gpasswd -a ${UNAME} audio

USER $UNAME
ENV HOME /home/${UNAME}
WORKDIR /home/${UNAME}
CMD ["bash"]
#RUN git clone https://github.com/diwic/alsa-rs.git
#RUN cd alsa-rs && cargo build
