import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart' show SynchronousFuture;

class MyLocalizations {
  MyLocalizations(this.locale);

  final Locale locale;

  static MyLocalizations? of(BuildContext context) {
    return Localizations.of<MyLocalizations>(context, MyLocalizations);
  }

  static Map<String, Map<String, String>> _localizedValues = {
    'en': {
      'asset_created': 'Asset created created or altered successfully.',
      'asset_deleted': 'Asset deleted successfully.',
      'asset': 'Asset',
      'assets': 'Assets',
      'closed': 'Closed',
      'comments': 'Comments',
      'comment_created': 'Comment created created or altered successfully.',
      'comment_deleted': 'Comment deleted successfully.',
      'content': 'Content',
      'creator': 'Creator',
      'description': 'Description',
      'edit_asset': 'Edit asset',
      'edit_comment': 'Edit comment',
      'edit_ticket': 'Edit ticket',
      'hostname': 'Server name',
      'new_asset': 'New asset',
      'new_comment': 'New comment',
      'new_ticket': 'New ticket',
      'no_assets': 'No assets',
      'pick_photo': 'Pick a photo',
      'please_enter_some_text': 'Please enter some text',
      'settings': 'Settings',
      'show_closed': 'Show closed tickets',
      'submit': 'SUBMIT',
      'ticket_created': 'Ticket created or altered successfully.',
      'ticket_deleted': 'Ticket deleted successfully.',
      'title': 'Title',
      'token': 'Token',
      'try_new_token': 'Error accessing data, please check your access token.'
    },
    'fr': {
      'asset_created': 'Actif créé ou modifié avec succès.',
      'asset_deleted': 'Actif supprimé avec succès.',
      'asset': 'Actif',
      'assets': 'Actifs',
      'closed': 'Fermé',
      'comments': 'Commentaires',
      'comment_created': 'Commentaire créé ou modifié avec succès.',
      'comment_deleted': 'Commentaire supprimé avec succès.',
      'content': 'Contenu',
      'creator': 'Auteur',
      'description': 'Description',
      'edit_asset': 'Modifier l\'actif',
      'edit_comment': 'Modifier le commentaire',
      'edit_ticket': 'Modifier le ticket',
      'hostname': 'Nom du serveur',
      'new_asset': 'Nouvel actif',
      'new_comment': 'Nouveau commentaire',
      'new_ticket': 'Nouveau ticket',
      'no_assets': 'Aucun actif',
      'pick_photo': 'Choisir une photo',
      'please_enter_some_text': 'Veuillez entrer du texte',
      'settings': 'Paramètres',
      'show_closed': 'Montrer les tickets fermés',
      'submit': 'VALIDER',
      'ticket_created': 'Ticket créé ou modifié avec succès.',
      'ticket_deleted': 'Ticket supprimé avec succès.',
      'title': 'Titre',
      'token': 'Jeton de sécurité',
      'try_new_token':
          'Erreur d\'accès aux données, veuillez vérifier votre jeton de sécurité.'
    },
  };

  String tr(String token) {
    return _localizedValues[locale.languageCode]![token] ?? token;
  }

  static String localizedValue(String locale, String token) {
    final lcl = ['en', 'fr'].contains(locale) ? locale : 'en';
    return _localizedValues[lcl]![token] ?? token;
  }
}

class MyLocalizationsDelegate extends LocalizationsDelegate<MyLocalizations> {
  const MyLocalizationsDelegate();

  @override
  bool isSupported(Locale locale) => ['en', 'fr'].contains(locale.languageCode);

  @override
  Future<MyLocalizations> load(Locale locale) {
    // Returning a SynchronousFuture here because an async "load" operation
    // isn't needed to produce an instance of MyLocalizations.
    return SynchronousFuture<MyLocalizations>(MyLocalizations(locale));
  }

  @override
  bool shouldReload(MyLocalizationsDelegate old) => false;
}
