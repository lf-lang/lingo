use crate::interface::Backend;
use crate::package::Package;

use crate::util;
use crate::util::command_line::run_and_capture;
use std::env;
use std::fs;
use std::process::Command;
use run_script::ScriptOptions;

pub struct Zephyr {
    package: Package,
}

impl Backend for Zephyr {
    fn from_package(package: &Package) -> Self {
        LFC {
            package: package.clone(),
        }
    }

    fn build(&self, config: &BuildArgs) -> bool {
        let reactor_copy = self.package.main_reactor.clone();

        let build_lambda = |main_reactor: &String| -> bool {
            println!("building main reactor: {}", &main_reactor);
            let (code, output, error) = run_script::run(
                r#"
                echo "Building Zephyr application"
                SCRIPT_DIR=$(dirname $0)

                set -e # Return on first error

                # Verify command line arguments
                if [ $# -lt 1 ]; then
                    echo "ERROR: Please pass the board to zephyr_build.sh. e.g. ./zephyr_build.sh qemu_cortex_m3 flash"
                    exit 1
                fi

                # Set some global variables
                export BOARD=$1
                export LF_SRC_DIRECTORY=$(pwd)

                # Replace reactor-c
                LFC_ROOT=$NGRES_ROOT/lingua-franca
                RC=$LFC_ROOT/org.lflang/src/lib/c/reactor-c

                cp -Lr $RC/core $LF_SOURCE_GEN_DIRECTORY/
                cp -Lr $RC/lib $LF_SOURCE_GEN_DIRECTORY/
                cp -Lr $RC/include $LF_SOURCE_GEN_DIRECTORY/

                # bash $LF_SRC_DIRECTORY/$SCRIPT_DIR/replace_reactor_c.sh .

                # Insert the zephyr find_package BEFORE project defintion. This important 
                #   see here: https://github.com/zephyrproject-rtos/zephyr/issues/18906?imz_s=ev33andn83vnopooshfrvkg7l6
                sed -i '/^project/i \
                find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE}) \
                ' $LF_SOURCE_GEN_DIRECTORY/CMakeLists.txt

                # Set the LF_QEMU_EMULATION flag if our board is qemu. It makes us NOT use HW timer but the sysclock which is less precise
                if [[ "$BOARD" == "qemu"* ]]; then 
                echo "Building for QEMU board"
                echo "
                target_compile_definitions(app PUBLIC LF_QEMU_EMULATION)
                " >> $LF_SOURCE_GEN_DIRECTORY/zephyr.cmake
                fi

                cp $LF_SRC_DIRECTORY/$SCRIPT_DIR/prj.conf $LF_SOURCE_GEN_DIRECTORY/
                cp $LF_SRC_DIRECTORY/$SCRIPT_DIR/Kconfig $LF_SOURCE_GEN_DIRECTORY/

                cd $LF_SOURCE_GEN_DIRECTORY

                # Build project
                west build -b $BOARD 

                if [[ "$2" == "flash" ]]; then

                if [[ "$BOARD" == "nrf"* ]]; then
                    echo "--- Flashing to NRF board"
                    # Flash application
                    bash $LF_SRC_DIRECTORY/$SCRIPT_DIR/zephyr_flash_nrf.sh .

                    # Debug application
                    # FIXME: Fix the issues here. Why isnt gdb working when invoked from this script?
                    # $LF_SRC_DIRECTORY/../scripts/zephyr_debug_nrf.sh
                elif [[ "$BOARD" == "qemu"* ]]; then 
                    echo "--- Executing on QEMU emulation"
                    west build -t run
                else
                    echo "Unrecognized board $BOARD" 
                    exit 1
                fi

                elif [[ "$2" == "debug" ]]; then

                if [[ "$BOARD" == "nrf"* ]]; then
                    echo "--- Debugging NRF board"

                    # Debug application
                    bash $LF_SRC_DIRECTORY/scripts/zephyr_debug_nrf.sh .

                fi

                fi

                "#,
                &args,
                &options,
            );

            //run_and_capture(&mut command).is_ok()
        };

        util::invoke_on_selected(binary, reactor_copy, build_lambda)
    }

    fn update(&self) -> bool {
        true
    }

    fn clean(&self) -> bool {
        println!("removing build artifacts in {:?}", env::current_dir());
        // just removes all the lingua-franca build artifacts
        fs::remove_dir_all("./bin").is_ok()
            && fs::remove_dir_all("./include").is_ok()
            && fs::remove_dir_all("./src-gen").is_ok()
            && fs::remove_dir_all("./lib64").is_ok()
            && fs::remove_dir_all("./share").is_ok()
            && fs::remove_dir_all("./build").is_ok()
    }
}
