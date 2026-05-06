FROM dxflrs/garage@sha256:866bd13ed2038ba7e7190e840482bc27234c4afaf77be8cfa439ae088c1e4690
COPY --from=busybox:musl /bin/busybox /bin/busybox
RUN ["/bin/busybox", "sh", "-c", "/bin/busybox --install /bin"]