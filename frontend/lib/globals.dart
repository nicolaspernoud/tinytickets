import 'package:flutter/foundation.dart';
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
    if (kIsWeb) {
      final token = Uri.base.queryParameters['token'];
      if (token != null) {
        App().prefs.setString("token", token);
      }
    }
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
      default:
        return Role.unknown;
    }
  }
}

enum Role {
  unknown,
  admin,
  user,
}
