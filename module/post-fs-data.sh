# Copyright 2023 shadow3aaa@gitbub.com
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# $1:value $2:path
lock_val() {
	umount $2
	chmod +w $2

	echo "$1" | tee /dev/fas_rs_mask $2
	/bin/find $2 -exec mount /dev/fas_rs_mask {} \;
	rm /dev/fas_rs_mask
}

lock_val "" "/odm/bin/hw/vendor.oplus.hardware.ormsHalService-aidl-service"
lock_val "" "/odm/bin/hw/vendor.oplus.hardware.urcc-service"
lock_val "" "/odm/bin/hw/vendor.oplus.hardware.gameopt-service"
