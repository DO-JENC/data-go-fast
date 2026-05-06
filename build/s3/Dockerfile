FROM busybox@sha256:19b646668802469d968a05342a601e78da4322a414a7c09b1c9ee25165042138 AS busybox

FROM dxflrs/garage@sha256:866bd13ed2038ba7e7190e840482bc27234c4afaf77be8cfa439ae088c1e4690
COPY --from=busybox /bin/busybox /bin/busybox
RUN ["/bin/busybox", "sh", "-c", "/bin/busybox --install /bin"]
