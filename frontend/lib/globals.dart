import 'package:shared_preferences/shared_preferences.dart';

class App {
  late SharedPreferences prefs;
  App._privateConstructor();

  static final App _instance = App._privateConstructor();

  factory App() {
    return _instance;
  }

  Future init() async {
    prefs = await SharedPreferences.getInstance();
  }

  Role get role {
    final token = prefs.getString("token") ?? "";
    final regex = RegExp(r'\$(.*)\$');
    if (!regex.hasMatch(token)) return Role.unknown;
    switch (regex.firstMatch(token)!.group(1)) {
      case "ADMIN":
        return Role.admin;
      case "USER":
        return Role.user;
      case "DESK":
        return Role.desk;
      default:
        return Role.unknown;
    }
  }
}

enum Role {
  unknown,
  admin,
  user,
  desk,
}
