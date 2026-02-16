# ============================================================================
# 자동 업데이트 도메인
# ============================================================================

self-update-checking = 업데이트 확인 중...
self-update-already-latest = 이미 최신 버전입니다 ({ $version })
self-update-new-version = 새 버전 사용 가능: { $current } → { $latest }
self-update-downloading = 업데이트 다운로드 중...
self-update-extracting = 업데이트 압축 해제 중...
self-update-installing = 업데이트 설치 중...
self-update-success = 버전 { $version }(으)로 성공적으로 업데이트되었습니다
self-update-error-platform = 지원되지 않는 운영 체제
self-update-error-arch = 지원되지 않는 아키텍처
self-update-error-no-asset = 플랫폼 { $platform }용 릴리스 자산을 찾을 수 없습니다
self-update-error-no-release = CLI 관리자 릴리스를 찾을 수 없습니다

# ============================================================================
# 셸 자동완성 도메인
# ============================================================================

completions-init-start = { $shell } 셸 자동완성 초기화 중...
completions-init-done = 완료! 자동완성이 설치되었습니다: { $path }
completions-restart-zsh = 셸을 다시 시작하거나 실행하세요: source ~/.zshrc
completions-restart-bash = 셸을 다시 시작하거나 실행하세요: source ~/.bashrc
completions-restart-fish = 자동완성이 새 fish 세션에서 즉시 활성화됩니다.
completions-restart-generic = 자동완성을 활성화하려면 셸을 다시 시작하세요.
completions-error-no-shell = 셸을 감지할 수 없습니다. 지정하세요: adi init bash|zsh|fish

# ============================================================================
# 플러그인 관리 도메인
# ============================================================================

# 플러그인 목록
plugin-list-title = 사용 가능한 플러그인:
plugin-list-empty = 레지스트리에 사용 가능한 플러그인이 없습니다.
plugin-installed-title = 설치된 플러그인:
plugin-installed-empty = 설치된 플러그인이 없습니다.
plugin-installed-hint = 플러그인 설치: adi plugin install <plugin-id>

# 플러그인 설치
plugin-install-downloading = { $id } v{ $version } ({ $platform }) 다운로드 중...
plugin-install-extracting = { $path }에 압축 해제 중...
plugin-install-success = { $id } v{ $version } 설치 완료!
plugin-install-already-installed = { $id } v{ $version }이(가) 이미 설치되어 있습니다
plugin-install-dependency = 의존성 설치 중: { $id }
plugin-install-error-platform = 플러그인 { $id }은(는) 플랫폼 { $platform }을(를) 지원하지 않습니다
plugin-install-pattern-searching = 패턴 "{ $pattern }"과 일치하는 플러그인 검색 중...
plugin-install-pattern-found = 패턴과 일치하는 { $count }개의 플러그인을 찾았습니다
plugin-install-pattern-none = "{ $pattern }"과 일치하는 플러그인을 찾을 수 없습니다
plugin-install-pattern-installing = { $count }개의 플러그인 설치 중...
plugin-install-pattern-success = { $count }개의 플러그인이 성공적으로 설치되었습니다!
plugin-install-pattern-failed = 설치 실패:

# 플러그인 업데이트
plugin-update-checking = { $id } 업데이트 확인 중...
plugin-update-already-latest = { $id }은(는) 이미 최신 버전입니다 ({ $version })
plugin-update-available = { $id }을(를) { $current }에서 { $latest }(으)로 업데이트 중...
plugin-update-downloading = { $id } v{ $version } 다운로드 중...
plugin-update-success = { $id }을(를) v{ $version }(으)로 업데이트했습니다
plugin-update-all-start = { $count }개의 플러그인 업데이트 중...
plugin-update-all-done = 업데이트 완료!
plugin-update-all-warning = { $id } 업데이트 실패: { $error }

# 플러그인 제거
plugin-uninstall-prompt = 플러그인 { $id }을(를) 제거하시겠습니까?
plugin-uninstall-cancelled = 취소되었습니다.
plugin-uninstall-progress = { $id } 제거 중...
plugin-uninstall-success = { $id }이(가) 성공적으로 제거되었습니다!
plugin-uninstall-error-not-installed = 플러그인 { $id }이(가) 설치되어 있지 않습니다

# ============================================================================
# 검색 도메인
# ============================================================================

search-searching = "{ $query }" 검색 중...
search-no-results = 결과를 찾을 수 없습니다.
search-packages-title = 패키지:
search-plugins-title = 플러그인:
search-results-summary = { $packages }개의 패키지와 { $plugins }개의 플러그인을 찾았습니다

# ============================================================================
# 서비스 도메인
# ============================================================================

services-title = 등록된 서비스:
services-empty = 등록된 서비스가 없습니다.
services-hint = 서비스를 추가하려면 플러그인 설치: adi plugin install <id>

# ============================================================================
# 실행 명령 도메인
# ============================================================================

run-title = 실행 가능한 플러그인:
run-empty = CLI 인터페이스가 있는 플러그인이 설치되어 있지 않습니다.
run-hint-install = 플러그인 설치: adi plugin install <plugin-id>
run-hint-usage = 플러그인 실행: adi run <plugin-id> [args...]
run-error-not-found = 플러그인 '{ $id }'을(를) 찾을 수 없거나 CLI 인터페이스가 없습니다
run-error-no-plugins = 실행 가능한 플러그인이 설치되어 있지 않습니다.
run-error-available = 실행 가능한 플러그인:
run-error-failed = 플러그인 실행 실패: { $error }

# ============================================================================
# 외부 명령 도메인
# ============================================================================

external-error-no-command = 명령이 제공되지 않았습니다
external-error-unknown = 알 수 없는 명령: { $command }
external-error-no-installed = 설치된 플러그인 명령이 없습니다.
external-hint-install = 플러그인 설치: adi plugin install <plugin-id>
external-available-title = 사용 가능한 플러그인 명령:
external-error-load-failed = 플러그인 '{ $id }' 로드 실패: { $error }
external-hint-reinstall = 재설치를 시도하세요: adi plugin install { $id }
external-error-run-failed = { $command } 실행 실패: { $error }

# 자동 설치
external-autoinstall-found = 플러그인 '{ $id }'이(가) 명령 '{ $command }'을(를) 제공합니다
external-autoinstall-prompt = 설치하시겠습니까? [y/N]
external-autoinstall-installing = 플러그인 '{ $id }' 설치 중...
external-autoinstall-success = 플러그인이 성공적으로 설치되었습니다!
external-autoinstall-failed = 플러그인 설치 실패: { $error }
external-autoinstall-disabled = 자동 설치가 비활성화되었습니다. 실행: adi plugin install { $id }
external-autoinstall-not-found = 명령 '{ $command }'을(를) 제공하는 플러그인을 찾을 수 없습니다

# ============================================================================
# 정보 명령
# ============================================================================

info-title = ADI CLI 정보
info-version = 버전
info-config-dir = 설정
info-plugins-dir = 플러그인
info-registry = 레지스트리
info-theme = 테마
info-language = 언어
info-installed-plugins = 설치된 플러그인 ({ $count })
info-no-plugins = 설치된 플러그인이 없습니다
info-commands-title = 명령
info-plugin-commands = 플러그인 명령:
info-cmd-info = CLI 정보, 버전, 경로 표시
info-cmd-start = 로컬 ADI 서버 시작
info-cmd-plugin = 플러그인 관리
info-cmd-run = 플러그인 CLI 실행
info-cmd-logs = 플러그인 로그 보기
info-cmd-self-update = adi CLI 업데이트

# ============================================================================
# 인터랙티브 명령 선택
# ============================================================================

interactive-select-command = 명령 선택

# 명령 라벨
interactive-cmd-info = 정보
interactive-cmd-start = 시작
interactive-cmd-plugin = 플러그인
interactive-cmd-search = 검색
interactive-cmd-run = 실행
interactive-cmd-logs = 로그
interactive-cmd-debug = 디버그
interactive-cmd-self-update = 자동 업데이트
interactive-cmd-completions = 자동완성
interactive-cmd-init = 초기화

# 명령 설명
interactive-cmd-info-desc = CLI 정보, 버전, 경로, 설치된 플러그인 표시
interactive-cmd-start-desc = 브라우저 연결을 위한 로컬 ADI 서버 시작
interactive-cmd-plugin-desc = 레지스트리에서 플러그인 관리
interactive-cmd-search-desc = 플러그인 및 패키지 검색
interactive-cmd-run-desc = 플러그인의 CLI 인터페이스 실행
interactive-cmd-logs-desc = 플러그인의 실시간 로그 스트리밍
interactive-cmd-debug-desc = 디버그 및 진단 명령
interactive-cmd-self-update-desc = adi CLI를 최신 버전으로 업데이트
interactive-cmd-completions-desc = 셸 자동완성 생성
interactive-cmd-init-desc = 셸 자동완성 초기화

# 인수 프롬프트
interactive-self-update-force = 최신 버전이어도 강제 업데이트하시겠습니까?
interactive-start-port = 포트
interactive-search-query = 검색어
interactive-completions-shell = 셸 선택
interactive-init-shell = 셸 선택 (자동 감지하려면 비워두세요)
interactive-logs-plugin-id = 플러그인 ID (예: adi.hive)
interactive-logs-follow = 로그 출력을 추적하시겠습니까?
interactive-logs-lines = 줄 수

# 플러그인 하위 명령
interactive-plugin-select = 플러그인 작업 선택
interactive-plugin-list = 사용 가능 목록
interactive-plugin-installed = 설치됨 목록
interactive-plugin-search = 검색
interactive-plugin-install = 설치
interactive-plugin-update = 업데이트
interactive-plugin-update-all = 모두 업데이트
interactive-plugin-uninstall = 제거
interactive-plugin-path = 경로 표시
interactive-plugin-install-id = 설치할 플러그인 ID (예: adi.tasks)
interactive-plugin-update-id = 업데이트할 플러그인 ID
interactive-plugin-uninstall-id = 제거할 플러그인 ID
interactive-plugin-path-id = 플러그인 ID

# ============================================================================
# 공통 메시지
# ============================================================================

common-version-prefix = v
common-tags-label = 태그:
common-error-prefix = 오류:
common-warning-prefix = 경고:
common-info-prefix = 정보:
common-success-prefix = 성공:
common-downloading-prefix = →
common-checkmark = ✓
common-arrow = →

# ============================================================================
# 오류 도메인
# ============================================================================

error-component-not-found = 컴포넌트 '{ $name }'을(를) 찾을 수 없습니다
error-installation-failed = '{ $component }' 설치 실패: { $reason }
error-dependency-missing = '{ $component }'에 필요한 종속성 '{ $dependency }'이(가) 설치되지 않았습니다
error-config = 구성 오류: { $detail }
error-io = IO 오류: { $detail }
error-serialization = 직렬화 오류: { $detail }
error-already-installed = 컴포넌트 '{ $name }'이(가) 이미 설치되어 있습니다
error-uninstallation-failed = '{ $component }' 제거 실패: { $reason }
error-registry = 레지스트리 오류: { $detail }
error-plugin-not-found = 플러그인을 찾을 수 없습니다: { $id }
error-plugin-host = 플러그인 호스트 오류: { $detail }
error-service = 서비스 오류: { $detail }
error-other = 오류: { $detail }
