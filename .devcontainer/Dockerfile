FROM ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive
ARG USER_UID=1000
ARG USER_GID=1000
ARG USERNAME=ubuntu

ENV LC_ALL=C.UTF-8
ENV LANG=C.UTF-8

# Arguments
ARG ESP_BOARD=all
ARG IDF_VERSION=v6.0.2

# Install dependencies
RUN apt-get update \
    && apt-get install -y pkg-config curl gcc clang libudev-dev unzip xz-utils \
    git wget flex bison gperf python3 python3-pip python3-venv cmake ninja-build ccache libffi-dev libssl-dev dfu-util libusb-1.0-0 \
    zsh neovim sccache \
    && apt-get clean -y && rm -rf /var/lib/apt/lists/* /tmp/library-scripts

RUN chsh -s /usr/bin/zsh ${USERNAME}
RUN echo 'ubuntu:ubuntu' | chpasswd
RUN usermod -aG sudo ubuntu
# Set users
USER ${USERNAME}
WORKDIR /home/${USERNAME}

# Install rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
    --default-toolchain none -y --profile minimal

# Update envs
ENV PATH=${PATH}:/home/${USERNAME}/.cargo/bin

# Install extra crates
RUN ARCH=$($HOME/.cargo/bin/rustup show | grep "Default host" | sed -e 's/.* //') && \
    curl -L "https://github.com/esp-rs/espup/releases/latest/download/espup-${ARCH}" -o "${HOME}/.cargo/bin/espup" && \
    chmod u+x "${HOME}/.cargo/bin/espup" && \
    curl -L "https://github.com/esp-rs/espflash/releases/latest/download/cargo-espflash-${ARCH}.zip" -o "${HOME}/.cargo/bin/cargo-espflash.zip" && \
    unzip "${HOME}/.cargo/bin/cargo-espflash.zip" -d "${HOME}/.cargo/bin/" && \
    rm "${HOME}/.cargo/bin/cargo-espflash.zip" && \
    chmod u+x "${HOME}/.cargo/bin/cargo-espflash" && \
    curl -L "https://github.com/esp-rs/espflash/releases/latest/download/espflash-${ARCH}.zip" -o "${HOME}/.cargo/bin/espflash.zip" && \
    unzip "${HOME}/.cargo/bin/espflash.zip" -d "${HOME}/.cargo/bin/" && \
    rm "${HOME}/.cargo/bin/espflash.zip" && \
    chmod u+x "${HOME}/.cargo/bin/espflash" && \
    curl -L "https://github.com/esp-rs/embuild/releases/latest/download/ldproxy-${ARCH}.zip" -o "${HOME}/.cargo/bin/ldproxy.zip" && \
    unzip "${HOME}/.cargo/bin/ldproxy.zip" -d "${HOME}/.cargo/bin/" && \
    rm "${HOME}/.cargo/bin/ldproxy.zip" && \
    chmod u+x "${HOME}/.cargo/bin/ldproxy" && \
    curl -L "https://github.com/esp-rs/esp-web-flash-server/releases/latest/download/web-flash-${ARCH}.zip" -o "${HOME}/.cargo/bin/web-flash.zip" && \
    unzip "${HOME}/.cargo/bin/web-flash.zip" -d "${HOME}/.cargo/bin/" && \
    rm "${HOME}/.cargo/bin/web-flash.zip" && \
    chmod u+x "${HOME}/.cargo/bin/web-flash"

# Install Xtensa Rust (with retry loop + --disable-timeouts to survive transient GitHub API 504s)
RUN for attempt in 1 2 3 4 5; do \
        echo "--- espup install attempt ${attempt}/5 ---"; \
        if ${HOME}/.cargo/bin/espup install \
            --targets "${ESP_BOARD}" \
            --log-level debug \
            --disable-timeouts \
            --export-file /home/${USERNAME}/export-esp.sh; then \
            exit 0; \
        fi; \
        echo "espup install failed (attempt ${attempt}/5), waiting 30s before retry..."; \
        sleep 30; \
    done; \
    echo "ERROR: espup install failed after 5 attempts"; \
    exit 1
RUN chown ${USERNAME} /home/${USERNAME}/export-esp.sh
# Set default toolchain
RUN rustup default esp

# Install ESP-IDF v6.0.2 (espup no longer installs ESP-IDF, so do it explicitly)
ENV IDF_PATH=/home/${USERNAME}/esp/esp-idf
ENV ESP_IDF_PATH=${IDF_PATH}

RUN git clone --recursive --depth 1 --shallow-submodules --single-branch \
        --branch ${IDF_VERSION} \
        https://github.com/espressif/esp-idf.git ${IDF_PATH} \
    && ${IDF_PATH}/install.sh ${ESP_BOARD}

# Make sure the ESP environment is active for bash sessions (devpod default shell)
RUN echo "source $HOME/export-esp.sh" >> ~/.bashrc \
    && echo "source ${IDF_PATH}/export.sh" >> ~/.bashrc

# Activate ESP environment
CMD [ "/bin/bash" ]


###################################
# Setup user env
###################################
RUN mkdir -p ~/.local &&  mkdir -p ~/.local/bin &&  mkdir -p ~/.cargo && mkdir -p ~/.cargo/bin

############################################
# ZSH
#
############################################
RUN cat <<EOF > /home/${USERNAME}/.zshrc
# History Settings
HISTFILE=~/.zsh_history
HISTSIZE=10000
SAVEHIST=10000
setopt appendhistory
setopt sharehistory

# Completion System
autoload -Uz compinit && compinit

# Prompt
PROMPT='%F{cyan}[%n@docker]%f %F{blue}%~%f %# '

# Aliases
alias ls='ls --color=auto'
alias ll='ls -lah'
export TERM='linux'
export PATH='$PATH:$HOME/.cargo/bin:$HOME/.local/bin'
. $HOME/.cargo/env
source $HOME/export-esp.sh
source $IDF_PATH/export.sh
EOF


