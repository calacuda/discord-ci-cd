ARG BASE_IMAGE

FROM dcicd-runner:latest AS runner
FROM $BASE_IMAGE

COPY --from=runner /usr/bin/dcicd-runner /usr/bin/dcicd-runner

RUN useradd -ms /bin/bash dcicd-runner

STOPSIGNAL SIGQUIT
# VOLUME ["/etc/", "/home/dcicd-runner"]
# ENTRYPOINT ["/usr/bin/dumb-init", "/entrypoint"]
# CMD ["run", "--user=gitlab-runner", "--working-directory=/home/gitlab-runner"]
USER dcicd-runner
WORKDIR /home/dcicd-runner
ENTRYPOINT ["/usr/bin/dcicd-runner"]
# CMD [ "./repo/" ]
# CMD ["run", "--user=dcicd-runner", "--working-directory=/home/dcicd-runner"]

