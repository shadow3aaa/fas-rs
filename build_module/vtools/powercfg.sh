#!/system/bin/sh
#
# Copyright 2023 shadow3aaa@gitbub.com
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

max_freq_per=/cache/fas_rs_nodes/max_freq_per

case "$1" in
"init" | "fast" | "pedestal") echo 100 >$max_freq_per ;;
"standby") echo 40 >$max_freq_per ;;
"powersave") echo 75 >$max_freq_per ;;
"balance") echo 85 >$max_freq_per ;;
"performance") echo 90 >$max_freq_per ;;
esac
