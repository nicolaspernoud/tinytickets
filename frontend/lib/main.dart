import 'package:flutter/material.dart';
import 'package:tinytickets/globals.dart';
import 'components/tickets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'i18n.dart';
import 'models/crud.dart';
import 'models/ticket.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await App().init();
  runApp(MyApp());
}

class MyApp extends StatelessWidget {
  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tiny Tickets',
      theme: ThemeData(
          colorScheme: ColorScheme.fromSeed(
            seedColor: Colors.amber,
          ),
          appBarTheme: AppBarTheme(
            backgroundColor: Colors.amber,
            elevation: 4,
            shadowColor: Theme.of(context).shadowColor,
          )),
      home: MyHomePage(title: 'Tiny Tickets'),
      localizationsDelegates: [
        const MyLocalizationsDelegate(),
        ...GlobalMaterialLocalizations.delegates,
        GlobalWidgetsLocalizations.delegate,
      ],
      supportedLocales: [
        const Locale('en', ''),
        const Locale('fr', ''),
      ],
    );
  }
}

class MyHomePage extends StatefulWidget {
  MyHomePage({Key? key, required this.title}) : super(key: key);
  final String title;
  @override
  _MyHomePageState createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  @override
  Widget build(BuildContext context) {
    return Tickets(crud: APICrud<Ticket>(), title: widget.title);
  }
}
