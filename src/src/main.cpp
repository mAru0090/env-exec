// DxLib + ImGUI 長めのテストコード (DxLib掲示板にてダウンロード可能な、暫定最新版必須 (該当掲示板題名: Direct3D等のバージョンについて 2025/04/09))
// main.cpp

// ==============================
// ==============================
// ==============================
// ヘッダファイルインクルード部
// ==============================
// ==============================
// ==============================

#include <algorithm>
#include <array>
#include <ctime>
#include <fstream>
#include <functional>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <stdint.h>
#include <string>
#include <vector>
#include <map>
#include <cmath>

#include <DxLib.h>
#include <d3d9.h>
#include <d3dx9tex.h>
#include <tchar.h>
#include <windows.h>

#include "imgui.h"
#include "imgui_impl_dx9.h"
#include "imgui_impl_win32.h"

// ================================
// ================================
// ================================
// 定数宣言部
// ================================
// ================================
// ================================

static const size_t maxLogSize = 2048;
static const int charWidth = 100;
static const int charHeight = 100;
static const int viewWidth = 1280;
static const int viewHeight = 800;
static const float root2 = 1.41421f;

// ================================
// ================================
// ================================
// 構造体・クラス・enum定義部
// ================================
// ================================
// ================================

// 仮想コンソール情報構造体
struct VisualConsoleData {
  std::array<std::string, maxLogSize> logMessages;
  size_t logIndex;
};

// メインコマンド情報構造体
struct CommandInfo {
  std::string name;
  std::vector<std::string> argTypes; // 例: {"int", "string"}
};

// ================================
// ================================
// ================================
// 関数宣言部
// ================================
// ================================
// ================================

void resetDevice();
void initializeDxLib();
void initializeImGui(HWND hwnd);
void finalizeImGui();
void finalizeDxLib();
void mainLoop();
void render();
void renderMainMenu();
void renderConsole();
void renderDebug();
void customImGuiStyle();
void commandCallBack(const std::string& cmd,const std::vector<std::string>& argTypes,const std::vector<std::string>& args);

// void renderImGuiTexture(unsigned int width, unsigned int height);

LRESULT WINAPI WndProc(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam);
// Forward declare message handler from imgui_impl_win32.cpp
extern IMGUI_IMPL_API LRESULT ImGui_ImplWin32_WndProcHandler(HWND hWnd,
                                                             UINT msg,
                                                             WPARAM wParam,
                                                             LPARAM lParam);
namespace DxLib {
// ＩＭＥメッセージのコールバック関数
extern LRESULT IMEProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam);
}; // namespace DxLib

// ================================
// ================================
// ================================
// 変数定義部
// ================================
// ================================
// ================================

static unsigned int whResizeWidth, whResizeHeight = 0;
static bool consoleModeSettingEnabled = true;
static bool debugModeSettingEnabled = true;
static bool exitFlag = false;
static VisualConsoleData consoleData; // 仮想コンソール情報用変数

// 固定コマンドと引数型の定義
static std::vector<CommandInfo> commands = {
    {"help", {}}, {"run", {"string"}}, {"exit", {}},
    {"now", {"string"}}, {"sleep", {"int"}}, {"date", {"string"}}, {"stop", {"string"}}, {"reset", {"string"}}};
static bool consoleWindowFocused = false;
static bool p0RunFlag,p0ResetFlag,p0StopFlag = false;

// static float p0X = (viewWidth - charWidth)/2.0f;
// static float p0Y = (viewWidth - charWidth)/2.0f;
// static int p0V = 4;

static float p0X = 0;
static float p0Y = 100;
static int p0V = 4;
static float fAngle =  DX_PI / 6.0f;
static float p0VX = p0V * cosf(fAngle);
static float p0VY = p0V * sinf(fAngle);
static float p0GR = 0;

// メイン関数
int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance,
                   LPSTR lpCmdLine, int nCmdShow) {
  // DxLibの初期化
  initializeDxLib();
  // ImGuiの初期化
  initializeImGui(static_cast<HWND>(GetMainWindowHandle()));
  // メインループ
  mainLoop();
  // ImGuiの終了処理
  finalizeImGui();
  // DxLibの終了処理
  finalizeDxLib();
  return 0;
}

// ================================
// ImGuiカスタムカラー設定関数
// ================================
void customImGuiStyle() {
  ImGuiStyle &style = ImGui::GetStyle();
  ImVec4 *colors = style.Colors;

  // 半透明な背景（ガラス風）
  colors[ImGuiCol_WindowBg] = ImVec4(0.10f, 0.11f, 0.13f, 0.85f);
  colors[ImGuiCol_ChildBg] = ImVec4(0.12f, 0.13f, 0.15f, 0.70f);
  colors[ImGuiCol_PopupBg] = ImVec4(0.14f, 0.15f, 0.17f, 0.80f);

  // アクティブ要素（くすみブルー）
  colors[ImGuiCol_Header] = ImVec4(0.20f, 0.33f, 0.45f, 0.55f);
  colors[ImGuiCol_HeaderHovered] = ImVec4(0.30f, 0.50f, 0.65f, 0.85f);
  colors[ImGuiCol_HeaderActive] = ImVec4(0.25f, 0.45f, 0.60f, 0.90f);

  // ボタン系（少し透明）
  colors[ImGuiCol_Button] = ImVec4(0.18f, 0.30f, 0.42f, 0.70f);
  colors[ImGuiCol_ButtonHovered] = ImVec4(0.26f, 0.45f, 0.60f, 0.90f);
  colors[ImGuiCol_ButtonActive] = ImVec4(0.20f, 0.40f, 0.55f, 1.00f);

  // フレーム背景（入力欄など）
  colors[ImGuiCol_FrameBg] = ImVec4(0.16f, 0.18f, 0.22f, 0.70f);
  colors[ImGuiCol_FrameBgHovered] = ImVec4(0.20f, 0.25f, 0.30f, 0.85f);
  colors[ImGuiCol_FrameBgActive] = ImVec4(0.25f, 0.32f, 0.40f, 1.00f);

  // タイトルバー（やや透過）
  colors[ImGuiCol_TitleBg] = ImVec4(0.08f, 0.09f, 0.10f, 0.80f);
  colors[ImGuiCol_TitleBgActive] = ImVec4(0.15f, 0.18f, 0.22f, 0.90f);
  colors[ImGuiCol_TitleBgCollapsed] = ImVec4(0.05f, 0.05f, 0.05f, 0.60f);

  // チェック・スライダー系アクセント
  colors[ImGuiCol_CheckMark] = ImVec4(0.42f, 0.80f, 0.60f, 1.00f);
  colors[ImGuiCol_SliderGrab] = ImVec4(0.35f, 0.70f, 0.55f, 1.00f);
  colors[ImGuiCol_SliderGrabActive] = ImVec4(0.30f, 0.60f, 0.50f, 1.00f);

  // 仕切りやグリップ
  colors[ImGuiCol_Separator] = ImVec4(0.30f, 0.30f, 0.30f, 0.30f);
  colors[ImGuiCol_ResizeGrip] = ImVec4(0.25f, 0.30f, 0.35f, 0.35f);
  colors[ImGuiCol_ResizeGripHovered] = ImVec4(0.35f, 0.45f, 0.50f, 0.55f);
  colors[ImGuiCol_ResizeGripActive] = ImVec4(0.40f, 0.55f, 0.60f, 0.85f);

  // タブ
  colors[ImGuiCol_Tab] = ImVec4(0.18f, 0.23f, 0.28f, 0.75f);
  colors[ImGuiCol_TabHovered] = ImVec4(0.28f, 0.40f, 0.50f, 0.90f);
  colors[ImGuiCol_TabActive] = ImVec4(0.23f, 0.33f, 0.42f, 1.00f);

  // スタイリング
  style.FrameRounding = 6.0f;
  style.GrabRounding = 5.0f;
  style.WindowRounding = 8.0f;
  style.ScrollbarRounding = 6.0f;
  style.TabRounding = 5.0f;

  style.WindowPadding = ImVec2(10, 10);
  style.FramePadding = ImVec2(6, 4);
  style.ItemSpacing = ImVec2(8, 6);
}

// ================================
// ImGuiの初期化関数
// ================================
void initializeImGui(HWND hwnd) {
  // Setup Dear ImGui context
  IMGUI_CHECKVERSION();
  ImGui::CreateContext();
  ImGuiIO &io = ImGui::GetIO();
  static_cast<void>(io);
  io.ConfigFlags |=
      ImGuiConfigFlags_NavEnableKeyboard; // Enable Keyboard Controls
  io.ConfigFlags |=
      ImGuiConfigFlags_NavEnableGamepad; // Enable Gamepad Controls
  ImFont *font =
      io.Fonts->AddFontFromFileTTF("c:\\windows\\fonts\\meiryo.ttc", 20.0f,
                                   nullptr, io.Fonts->GetGlyphRangesJapanese());

  IM_ASSERT(font != nullptr);

  // Setup Dear ImGui style
  // ImGui::StyleColorsLight();
  // ImGui::StyleColorsDark();
  customImGuiStyle();

  // Setup Platform/Renderer backends
  ImGui_ImplWin32_Init(hwnd);
  IDirect3DDevice9 *pDevice = static_cast<IDirect3DDevice9 *>(
      const_cast<void *>(GetUseDirect3DDevice9()));
  if (!pDevice) {
    std::cerr << "Failed to GetUseDirect3DDevice9(): returned nullptr"
              << std::endl;
    return;
  }
  ImGui_ImplDX9_Init(pDevice);
}
// ================================
// ImGuiの終了処理関数
// ================================
void finalizeImGui() {
  // Cleanup
  ImGui_ImplDX9_Shutdown();
  ImGui_ImplWin32_Shutdown();
  ImGui::DestroyContext();
}

// ================================
// DxLibの初期化関数
// ================================
void initializeDxLib() {
  static const unsigned int whWidth = 1280;
  static const unsigned int whHeight = 800;
  static const unsigned int refreshRate = 120;
  static const unsigned int colorBitDepth = 32;
  SetUseCharCodeFormat(DX_CHARCODEFORMAT_UTF8);

  ChangeWindowMode(TRUE);
  SetGraphMode(whWidth, whHeight, colorBitDepth, refreshRate);
  SetWindowSize(whWidth, whHeight);
  SetUseDirect3D9Ex(FALSE);
  // SetUseTSFFlag( FALSE ) ;
  // SetUseIMEFlag( TRUE ) ;
  SetUseDirect3DVersion(DX_DIRECT3D_9);
  if (DxLib_Init() == -1) {
    std::cerr << "Failed to DxLib_Init()" << std::endl;
    return;
  }
  SetDrawMode(DX_DRAWMODE_NEAREST); // ドット感とか滑らか制御（お好み）
  SetUseZBuffer3D(TRUE);
  SetWriteZBuffer3D(TRUE);
  SetDrawScreen(DX_SCREEN_BACK);
  SetMouseDispFlag(TRUE);
  SetHookWinProc(WndProc);
}

// ================================
// DxLibの終了処理関数
// ================================
void finalizeDxLib() {
  if (DxLib_End() == -1) {
    std::cerr << "Failed to DxLib_End()" << std::endl;
    return;
  }
}

// ================================
// メインループ関数
// ================================
void mainLoop() {

  IDirect3DDevice9 *pDevice = static_cast<IDirect3DDevice9 *>(
      const_cast<void *>(GetUseDirect3DDevice9()));
  int leftKey,rightKey,upKey,downKey = 0;
  while (ProcessMessage() == 0) {
    SetDrawMode(DX_DRAWMODE_BILINEAR); // 補完モード
    if (exitFlag)
      break;
    // if ( CheckHitKey( KEY_INPUT_ESCAPE ) > 0 ) break;
    ClearDrawScreen();
    // リサイズ処理
    if (whResizeWidth != 0 && whResizeHeight != 0) {
      SetChangeScreenModeGraphicsSystemResetFlag(FALSE);
      SetGraphMode(whResizeWidth, whResizeHeight, 120, 32);
      SetChangeScreenModeGraphicsSystemResetFlag(TRUE);
      whResizeWidth = whResizeHeight = 0;
      resetDevice();
    }
    if (!p0StopFlag&&p0RunFlag) { 
    	/*
    	  leftKey = CheckHitKey(KEY_INPUT_LEFT);
    	  rightKey = CheckHitKey(KEY_INPUT_RIGHT);
    	  upKey = CheckHitKey(KEY_INPUT_UP);
    	  downKey = CheckHitKey(KEY_INPUT_DOWN);
    	  int horizontal = (leftKey > 0) + (rightKey > 0);
	  int vertical   = (upKey > 0) + (downKey > 0);
          bool diagonalMove = (horizontal == 1 && vertical == 1);

    	  if (leftKey) {
    	    p0X -= diagonalMove ? (p0V / root2) : p0V;
	  }
	  if (rightKey) {
            p0X += diagonalMove ? (p0V / root2) : p0V;
	  }
	  if (upKey) {
    	    p0Y -= diagonalMove ? (p0V / root2) : p0V;-++
	  }
	  if (downKey) {
    	    p0Y += diagonalMove ? (p0V / root2) : p0V;
	  }
	*/

    	p0X += p0VX;
    	p0Y += p0VY;
    	
    	
   
    }
    if (p0ResetFlag) {
      p0X = 0;
      p0Y = 100;
    }
    render();

    ScreenFlip();
  }
}

// ================================
// メインレンダリング関数
// ================================
void render() {
  RenderVertex(); // DxLib描画に被らないようにする
  // ImGuiの描画を開始
  ImGui_ImplDX9_NewFrame();
  ImGui_ImplWin32_NewFrame();
  ImGui::NewFrame();
  
  DrawBox(p0X,p0Y,p0X+charWidth,p0Y+charWidth,GetColor(255,255,255),TRUE);
  // 各ウィンドウ描画
  
  renderMainMenu();
  if (consoleModeSettingEnabled)
    renderConsole();
  if (debugModeSettingEnabled)
    renderDebug();
  // ImGuiの描画を終了
  ImGui::EndFrame();
  ImGui::Render();
  ImGui_ImplDX9_RenderDrawData(ImGui::GetDrawData());
  RefreshDxLibDirect3DSetting(); // Direct3Dの設定をクリア
}

// ================================
// メインメニューレンダリング関数
// ================================
void renderMainMenu() {
  ImGuiIO &io = ImGui::GetIO();
  if (io.KeyCtrl && ImGui::IsKeyPressed(ImGuiKey_O, false)) {
    // Ctrl+Oが押された処理
  }
  if (io.KeyCtrl && ImGui::IsKeyPressed(ImGuiKey_S, false)) {
    // Ctrl+Sが押された処理
  }
  if (io.KeyCtrl && ImGui::IsKeyPressed(ImGuiKey_Q, false)) {
    // Ctrl+Qが押された処理
    exitFlag = true;
  }
  if (io.KeyCtrl && ImGui::IsKeyPressed(ImGuiKey_Z, false)) {
    // Ctrl+Zが押された処理
  }

  if (ImGui::BeginMainMenuBar()) {
    // --- File メニュー ---
    if (ImGui::BeginMenu("File")) {
      // Open
      bool openClicked = ImGui::MenuItem("Open", "Ctrl+O");
      if (ImGui::IsItemHovered()) {
        ImGui::BeginTooltip();
        ImGui::Text("===ファイル->開く===\n");
        ImGui::Text("主に拡張子(*.png/*.jpg/*.bmp/"
                    "*.webp)等のファイルを読み込みます");
        ImGui::EndTooltip();
      }
      if (openClicked) {
        // Open の処理
      }

      // Save
      bool saveClicked = ImGui::MenuItem("Save", "Ctrl+S");
      if (ImGui::IsItemHovered()) {
        ImGui::BeginTooltip();
        ImGui::Text("===ファイル->保存===\n");
        ImGui::Text("現在の状態を保存します");
        ImGui::EndTooltip();
      }
      if (saveClicked) {
        // Save の処理
      }

      // Quit
      bool quitClicked = ImGui::MenuItem("Quit", "Ctrl+Q");
      if (ImGui::IsItemHovered()) {
        ImGui::BeginTooltip();
        ImGui::Text("===ファイル->終了===\n");
        ImGui::Text("アプリケーションを終了します");
        ImGui::EndTooltip();
      }
      if (quitClicked) {
        exitFlag = true;
      }

      ImGui::EndMenu();
    }

    // --- Edit メニュー ---
    if (ImGui::BeginMenu("Edit")) {
      bool undoClicked = ImGui::MenuItem("Undo", "Ctrl+Z");
      if (ImGui::IsItemHovered()) {
        ImGui::BeginTooltip();
        ImGui::Text("===編集->元に戻す===\n");
        ImGui::Text("直前の操作を取り消します");
        ImGui::EndTooltip();
      }
      if (undoClicked) {
        // Undoの処理
      }
      ImGui::EndMenu();
    }

    // --- Config メニュー ---
    if (ImGui::BeginMenu("Config")) {
      ImGui::MenuItem("Console", nullptr, &consoleModeSettingEnabled);
      if (ImGui::IsItemHovered()) {
        ImGui::BeginTooltip();
        ImGui::Text("===設定->コンソール===\n");
        ImGui::Text("コンソール表示のオンオフを切り替えます");
        ImGui::EndTooltip();
      }
      ImGui::MenuItem("Debug", nullptr, &debugModeSettingEnabled);
      if (ImGui::IsItemHovered()) {
        ImGui::BeginTooltip();
        ImGui::Text("===設定->デバッグ===\n");
        ImGui::Text("デバッグ表示のオンオフを切り替えます");
        ImGui::EndTooltip();
      }
      ImGui::EndMenu();
    }

    ImGui::EndMainMenuBar();
  }
}

// ================================
// 仮想コンソールレンダリング関数
// ================================
void renderConsole() {
  ImGui::PushStyleColor(ImGuiCol_WindowBg, ImVec4(0.12f, 0.12f, 0.18f, 1.00f));
  ImGui::PushStyleColor(ImGuiCol_Border, ImVec4(0.8f, 0.8f, 0.8f, 0.7f));
  ImGui::PushStyleVar(ImGuiStyleVar_WindowBorderSize, 1.5f);

  ImGuiIO &io = ImGui::GetIO();
  ImVec2 consoleSize(io.DisplaySize.x, 250);
  ImVec2 consolePos(0, io.DisplaySize.y - consoleSize.y);
  ImGui::SetNextWindowPos(consolePos);
  ImGui::SetNextWindowSize(consoleSize);

  if (ImGui::Begin("Console")) {
    if (ImGui::IsWindowFocused()) {
      if (ImGui::IsItemHovered()) {
      	consoleWindowFocused = true;
      } else {
      	consoleWindowFocused = false;
      }
    }
    if(consoleWindowFocused) {
        ImGui::BeginTooltip();
        ImGui::Text("===仮想コンソール===\n");
        ImGui::Text(
            "このウィンドウでは、エディタ上のログやその他コマンド等を実行"
            "することができます。");
        ImGui::EndTooltip();
    }
    static std::array<char, 256> inputBuf = {0}; // 入力バッファ

    ImGui::Separator();

    // ログ表示
    ImGui::BeginChild("LogRegion",
                      ImVec2(0, -2 * ImGui::GetFrameHeightWithSpacing()), false,
                      ImGuiWindowFlags_HorizontalScrollbar);

    for (size_t i = 0; i < consoleData.logIndex; ++i) {
      const std::string &fullMsg = consoleData.logMessages[i];
      std::istringstream iss(fullMsg);
      std::string line;
      while (std::getline(iss, line)) {
        if (line.find("[ERROR]Not found") != std::string::npos) {
          ImGui::TextColored(ImVec4(1.0f, 0.3f, 0.3f, 1.0f), "%s",
                             line.c_str());
        } else if (line.find("[NOTE]Did you mean?") != std::string::npos) {
          ImGui::TextColored(ImVec4(0.4f, 1.0f, 0.4f, 1.0f), "%s",
                             line.c_str());
        } else {
          ImGui::TextColored(ImVec4(1.0f, 1.0f, 1.0f, 1.0f), "%s",
                             line.c_str());
        }
      }
    }

    if (ImGui::GetScrollY() >= ImGui::GetScrollMaxY()) {
      ImGui::SetScrollHereY(0.8f);
    }
    ImGui::EndChild();

    ImGui::Separator();

    // 入力処理関数
    auto processInput = [&]() {
      std::string input(inputBuf.data());
      if (!input.empty()) {
        input = input.substr(input.find_first_not_of(" \t\r\n"));
        input = input.substr(0, input.find_last_not_of(" \t\r\n") + 1);
      }

      std::cout << "input: " << input << std::endl;

      std::istringstream iss(input);
      std::string cmd;
      iss >> cmd;

      auto it =
          std::find_if(commands.begin(), commands.end(),
                       [&](const CommandInfo &c) { return c.name == cmd; });

      if (it == commands.end()) {
        std::string message = "[ERROR]Not found command: " + cmd;

        // 部分一致候補を探す（2文字以上）
        std::vector<std::string> suggestions;
        for (const auto &c : commands) {
          if (cmd.size() >= 2 && c.name.substr(0, cmd.size()) == cmd) {
            suggestions.push_back(c.name);
          }
        }

        if (!suggestions.empty()) {
          message += "\n[NOTE]Did you mean?";
          for (const auto &s : suggestions) {
            message += "\n  - " + s;
          }
        }

        consoleData.logMessages[consoleData.logIndex++ %
                                consoleData.logMessages.size()] = message;
        return;
      }

      std::vector<std::string> args;
      std::string arg;
      while (iss >> arg)
        args.push_back(arg);

      if (args.size() != it->argTypes.size()) {
        consoleData.logMessages[consoleData.logIndex++ %
                                consoleData.logMessages.size()] =
            "Argument count mismatch for command: " + cmd;
        return;
      }
      commandCallBack(cmd,it->argTypes,args);
      inputBuf.fill(0); // 入力欄クリア
    };

    // 入力欄とボタンUI
    bool enterPressed =
        ImGui::InputText("Input", inputBuf.data(), inputBuf.size(),
                         ImGuiInputTextFlags_EnterReturnsTrue);
    ImGui::SameLine();
    if (ImGui::Button("Enter") || enterPressed) {
      processInput();
    }
    ImGui::SameLine();
    if (ImGui::Button("Clear")) {
      consoleData.logMessages.fill("");
      consoleData.logIndex = 0;
    }
  }
  ImGui::PopStyleColor(2);
  ImGui::PopStyleVar(1);
  ImGui::End();
}

// ================================
// デバッグ用ウィンドウレンダリング関数
// ================================
void renderDebug() {
  if (ImGui::Begin("Debug")) {
    const std::string debugP0Message = "p0 x: " + std::to_string(p0X) + " y: " + std::to_string(p0Y) + " v: " + std::to_string(p0V) + " vx: "+std::to_string(p0VX) + " vy: " + std::to_string(p0VY) ;
    ImGui::Text(debugP0Message.c_str());  
  }
  ImGui::End();
}
// ================================
// 実行コマンドコールバック関数
// ================================
void commandCallBack(const std::string& cmd,const std::vector<std::string>& argTypes,const std::vector<std::string>& args) {
      
      // 型チェック（int と string のみ対応）
      for (size_t i = 0; i < args.size(); ++i) {
        const std::string &type = argTypes[i];
        if (type == "int") {
          try {
            std::stoi(args[i]);
          } catch (...) {
            consoleData.logMessages[consoleData.logIndex++ %
                                    consoleData.logMessages.size()] =
                "Argument " + std::to_string(i) + " must be int.";
            return;
          }
        }
        // string型は何でもOKなのでスキップ
      }

      // 実行ログ表示
      std::string message = "Executed command: " + cmd;
      for (const auto &a : args)
        message += " [" + a + "]";
      consoleData.logMessages[consoleData.logIndex++ %
                              consoleData.logMessages.size()] = message;

      // 個別処理例：help
      if (cmd == "help") {
        const std::vector<std::string> helpMessage = {
            "HELP:",
            "Available commands:",
            " - help",
            " - run <int>(animation id)",
            " - exit",
            " - now <string>(none)",
            " - sleep <int>(milli duration)",
            " - date <string>(format)"};
        for (const auto &msg : helpMessage) {
          consoleData.logMessages[consoleData.logIndex++ %
                                  consoleData.logMessages.size()] = msg;
        }
      } else if (cmd == "exit") {
        exitFlag = true;
      } else if (cmd == "sleep") {
        const int arg0 = std::stoi(args[0]);
        const std::string msg = "sleep milli times: " + std::to_string(arg0);
        consoleData.logMessages[consoleData.logIndex++ %
                                consoleData.logMessages.size()] = msg;
        Sleep(arg0);
      } else if (cmd == "date") {
        const std::string arg0 = args[0];
        std::time_t now = std::time(nullptr);
        std::tm *local = std::localtime(&now);
        std::ostringstream oss;
        oss << std::put_time(local, arg0.c_str());
        const std::string msg = "date: " + oss.str();
        consoleData.logMessages[consoleData.logIndex++ %
                                consoleData.logMessages.size()] = msg;
      } else if (cmd == "run") {
      	const std::string arg0 = args[0];
      	if (arg0 == "p0") {
		p0RunFlag = true;	
		p0StopFlag = false;	
		p0ResetFlag = false;	
      	}
      } else if (cmd == "stop") {
      	const std::string arg0 = args[0];
      	if (arg0 == "p0") {
		p0StopFlag = true;	
		p0RunFlag = false;
		p0ResetFlag = false;	
      	}
      } else if (cmd == "reset") {
      	const std::string arg0 = args[0];
      	if (arg0 == "p0") {
		p0ResetFlag = true;	
		p0RunFlag = false;	
		p0StopFlag = false;	
      	}
      }

}

// ================================
// DirectX9デバイスのリセットをする関数
// ================================
void resetDevice() {
  ImGui_ImplDX9_InvalidateDeviceObjects();
  IDirect3DDevice9 *pDevice = static_cast<IDirect3DDevice9 *>(
      const_cast<void *>(GetUseDirect3DDevice9()));
  if (!pDevice) {
    std::cerr << "Failed to GetUseDirect3DDevice9(): returned nullptr"
              << std::endl;
    return;
  }
  HRESULT hr = pDevice->Reset(nullptr);
  if (hr == D3DERR_INVALIDCALL)
    IM_ASSERT(0);
  ImGui_ImplDX9_CreateDeviceObjects();
}

// Win32 メッセージプロージャ関数
LRESULT WINAPI WndProc(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam) {
  if (ImGui_ImplWin32_WndProcHandler(hWnd, msg, wParam, lParam)) {
    SetUseHookWinProcReturnValue(TRUE);
    return true;
  }

  switch (msg) {
  case WM_IME_SETCONTEXT:
  case WM_IME_STARTCOMPOSITION:
  case WM_IME_ENDCOMPOSITION:
  case WM_IME_COMPOSITION:
  case WM_IME_NOTIFY:
  case WM_IME_REQUEST:
    SetUseHookWinProcReturnValue(TRUE);
    // return DefWindowProc(hWnd, msg, wParam, lParam);
    return IMEProc(hWnd, msg, wParam, lParam);
  case WM_SYSCOMMAND:
    if ((wParam & 0xfff0) == SC_KEYMENU) { // Disable ALT application menu
      SetUseHookWinProcReturnValue(TRUE);
      return 0;
    }
    break;
  }

  return 0;
}