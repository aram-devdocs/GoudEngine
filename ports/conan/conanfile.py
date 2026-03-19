import os

from conan import ConanFile
from conan.errors import ConanException
from conan.tools.files import copy, get
from conan.tools.layout import basic_layout


class GoudEngineConan(ConanFile):
    name = "goud-engine"
    description = "GoudEngine C/C++ SDK — cross-platform game engine"
    license = "MIT"
    homepage = "https://github.com/aram-devdocs/GoudEngine"
    url = "https://github.com/aram-devdocs/GoudEngine"
    topics = ("game-engine", "graphics", "ecs")
    package_type = "shared-library"
    settings = "os", "arch", "compiler", "build_type"

    def _rid(self):
        os_name = str(self.settings.os)
        arch = str(self.settings.arch)
        if os_name == "Linux" and arch == "x86_64":
            return "linux-x64", "libgoud_engine.so"
        elif os_name == "Macos" and arch == "x86_64":
            return "osx-x64", "libgoud_engine.dylib"
        elif os_name == "Macos" and arch == "armv8":
            return "osx-arm64", "libgoud_engine.dylib"
        elif os_name == "Windows" and arch == "x86_64":
            return "win-x64", "goud_engine.dll"
        else:
            raise ConanException(f"Unsupported platform: {os_name}-{arch}")

    def layout(self):
        basic_layout(self)

    def source(self):
        rid, _ = self._rid()
        tarball = f"goud-engine-v{self.version}-{rid}.tar.gz"
        url = f"https://github.com/aram-devdocs/GoudEngine/releases/download/v{self.version}/{tarball}"
        get(self, url, strip_root=True)

    def package(self):
        rid, lib_name = self._rid()
        src = self.source_folder

        copy(self, "*.h", src=os.path.join(src, "include"), dst=os.path.join(self.package_folder, "include"), keep_path=True)
        copy(self, "*.hpp", src=os.path.join(src, "include"), dst=os.path.join(self.package_folder, "include"), keep_path=True)
        copy(self, lib_name, src=os.path.join(src, "lib"), dst=os.path.join(self.package_folder, "lib"))
        copy(self, "*.cmake", src=os.path.join(src, "cmake"), dst=os.path.join(self.package_folder, "cmake"))
        copy(self, "LICENSE", src=src, dst=os.path.join(self.package_folder, "licenses"), keep_path=False)

    def package_info(self):
        self.cpp_info.libs = ["goud_engine"]
        self.cpp_info.set_property("cmake_file_name", "GoudEngine")
        self.cpp_info.set_property("cmake_target_name", "GoudEngine::GoudEngine")
